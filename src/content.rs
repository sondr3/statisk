use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use minijinja::{context, value::Value};
use serde::Serialize;

use crate::{
    context::StatiskContext as SContext,
    frontmatter::Frontmatter,
    jotdown::render_jotdown,
    templating::{TemplatePath, create_base_context},
    typst::render_typst,
    utils::{split_frontmatter, unprefixed_parent},
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, Serialize)]
pub enum ContentType {
    HTML,
    XML,
    Jotdown,
    Typst,
    Unknown,
}

impl ContentType {
    pub fn from_ext(path: &Path) -> Result<Self> {
        match path.extension() {
            None => bail!("No extension for content type"),
            Some(kind) => match kind.to_string_lossy().to_string().as_ref() {
                "xml" | "xsl" => Ok(ContentType::XML),
                "html" => Ok(ContentType::HTML),
                "typ" => Ok(ContentType::Typst),
                "dj" => Ok(ContentType::Jotdown),
                _ => Ok(ContentType::Unknown),
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Content {
    pub source: PathBuf,
    pub out_path: PathBuf,
    pub dir: Option<String>,
    pub url: String,
    pub frontmatter: Frontmatter,
    pub content: String,
    pub kind: ContentType,
}

impl Content {
    pub fn from_path(path: &Path, root: &Path, kind: ContentType) -> Result<Self> {
        let file = std::fs::read_to_string(path)?;
        let stem = path.file_stem().unwrap().to_string_lossy();
        let stem = stem.as_ref();

        let (frontmatter, content) =
            split_frontmatter(&file).ok_or(anyhow!("Could not find content or frontmatter"))?;

        let frontmatter = match (kind, frontmatter) {
            (ContentType::XML, None) => Frontmatter::empty(),
            (_, Some(fm)) => Frontmatter::deserialize(&fm)?,
            _ => bail!("Missing frontmatter in content"),
        };

        let dir = unprefixed_parent(path, root);
        let out_path = out_path(kind, path, dir.as_ref(), stem, &frontmatter);

        let url = frontmatter.url(
            &out_path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        );

        Ok(Content {
            source: path.to_path_buf(),
            url,
            kind,
            out_path,
            dir,
            content,
            frontmatter,
        })
    }

    pub fn render(&self, context: &SContext) -> Result<String> {
        match self.kind {
            ContentType::HTML | ContentType::XML => self.render_template(context),
            ContentType::Jotdown | ContentType::Typst => self.render_content(context),
            ContentType::Unknown => bail!("Cannot render unknown files"),
        }
    }

    pub fn filename(&self) -> String {
        self.source.file_name().map_or_else(
            || panic!("No filename found"),
            |name| name.to_string_lossy().to_string(),
        )
    }

    pub fn is_public_page(&self) -> bool {
        matches!(self.kind, ContentType::Jotdown | ContentType::HTML) && !self.is_special_page()
    }

    pub fn context(&self, context: &SContext) -> Result<Value> {
        let content = match self.kind {
            ContentType::Jotdown => render_jotdown(&self.content)?,
            ContentType::Typst => render_typst(&self.content)?,
            _ => self.content.clone(),
        };
        let frontmatter_context = self.frontmatter.to_context();

        Ok(context! {
            ..frontmatter_context,
            ..context! {
                content => content,
                canonical_url => context.statisk.url().join(&self.url)?,
            }
        })
    }

    fn is_special_page(&self) -> bool {
        self.out_path == PathBuf::from("404.html") || self.out_path == PathBuf::from("500.html")
    }

    fn layout(&self) -> TemplatePath {
        match &self.frontmatter.layout {
            Some(layout) => TemplatePath(None, layout.to_string()),
            None => TemplatePath(self.dir.clone(), "page".to_string()),
        }
    }

    fn render_content(&self, app_context: &SContext) -> Result<String> {
        let base_context = create_base_context(app_context);
        let context = self.context(app_context)?;
        let context = context! { ..base_context, ..context };
        app_context
            .templates
            .render_template(&self.layout(), context)
    }

    fn render_template(&self, app_context: &SContext) -> Result<String> {
        let base_context = create_base_context(app_context);
        let context = self.context(app_context)?;
        let context = context! { ..base_context, ..context };
        let env = app_context.templates.environment.acquire_env()?;
        let template = env.template_from_str(&self.content)?;
        template.render(context).context("Could not render")
    }
}

fn out_path(
    kind: ContentType,
    path: &Path,
    dir: Option<&String>,
    stem: &str,
    frontmatter: &Frontmatter,
) -> PathBuf {
    match kind {
        ContentType::XML => PathBuf::from(path.file_name().unwrap_or_default()),
        ContentType::HTML | ContentType::Unknown => {
            // First check if this is a special page (like 404.html)
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename == "404.html" || filename == "500.html" {
                    return PathBuf::from(filename);
                }
            }

            // Then handle regular pages
            match (dir, &frontmatter.slug) {
                (None, None) => PathBuf::from("index.html"),
                (None, Some(slug)) => [slug, "index.html"].into_iter().collect(),
                (Some(dir), Some(slug)) => [dir, slug, "index.html"].into_iter().collect(),
                (Some(dir), None) => [dir, "index.html"].into_iter().collect(),
            }
        }
        ContentType::Jotdown | ContentType::Typst => match &frontmatter.slug {
            Some(slug) => [slug, "index.html"].into_iter().collect(),
            None => [stem, "index.html"].into_iter().collect(),
        },
    }
}
