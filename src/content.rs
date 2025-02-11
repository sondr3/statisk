use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use minijinja::{context, value::Value};
use serde::Serialize;

use crate::{
    context::Context as SContext,
    frontmatter::Frontmatter,
    jotdown::render_jotdown,
    templating::{create_base_context, TemplatePath},
    utils::{split_frontmatter, unprefixed_parent},
    BuildMode,
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, Serialize)]
pub enum ContentType {
    HTML,
    XML,
    Unknown,
    Jotdown,
}

impl ContentType {
    pub fn from_ext(path: &Path) -> Result<Self> {
        match path.extension() {
            None => bail!("No extension for content type"),
            Some(kind) => match kind.to_string_lossy().to_string().as_ref() {
                "xml" | "xsl" => Ok(ContentType::XML),
                "html" => Ok(ContentType::HTML),
                _ => Ok(ContentType::Unknown),
            },
        }
    }
}

#[derive(Debug, Serialize)]
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
        let out_path: PathBuf = match kind {
            ContentType::XML => PathBuf::from(path.file_name().unwrap_or_default()),
            ContentType::HTML | ContentType::Unknown => match (&dir, &frontmatter.slug) {
                (None, None) => PathBuf::from("index.html"),
                (None, Some(slug)) => [slug, "index.html"].into_iter().collect(),
                (Some(dir), Some(slug)) => [dir, slug, "index.html"].into_iter().collect(),
                (Some(dir), None) => [dir, "index.html"].into_iter().collect(),
            },
            ContentType::Jotdown => match &frontmatter.slug {
                Some(slug) => [slug, "index.html"].into_iter().collect(),
                None => [stem, "index.html"].into_iter().collect(),
            },
        };

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

    pub fn render(&self, mode: BuildMode, context: &SContext) -> Result<String> {
        match self.kind {
            ContentType::HTML | ContentType::XML => self.render_template(mode, context),
            ContentType::Jotdown => self.render_jotdown(mode, context),
            ContentType::Unknown => bail!("Cannot render unknown files"),
        }
    }

    pub fn filename(&self) -> String {
        self.source.file_stem().map_or_else(
            || panic!("No filename found"),
            |name| name.to_string_lossy().to_string(),
        )
    }

    pub fn is_page(&self) -> bool {
        matches!(self.kind, ContentType::Jotdown | ContentType::HTML)
    }

    fn layout(&self) -> TemplatePath {
        match &self.frontmatter.layout {
            Some(layout) => TemplatePath(None, layout.to_string()),
            None => TemplatePath(self.dir.clone(), "page".to_string()),
        }
    }

    fn render_jotdown(&self, mode: BuildMode, app_context: &SContext) -> Result<String> {
        let base_context = create_base_context(mode, app_context);
        let context = self.context(app_context)?;
        let context = context! { ..base_context, ..context };
        app_context
            .templates
            .render_template(&self.layout(), context)
    }

    fn render_template(&self, mode: BuildMode, app_context: &SContext) -> Result<String> {
        let base_context = create_base_context(mode, app_context);
        let context = self.context(app_context)?;
        let context = context! { ..base_context, ..context };
        let env = app_context.templates.environment.acquire_env()?;
        let template = env.template_from_str(&self.content)?;
        template.render(context).context("Could not render")
    }

    pub fn context(&self, context: &SContext) -> Result<Value> {
        let content = match self.kind {
            ContentType::Jotdown => render_jotdown(&self.content)?,
            _ => self.content.clone(),
        };
        let frontmatter_context = self.frontmatter.to_context();

        Ok(context! {
            ..frontmatter_context,
            ..context! {
                content => content,
                canonical_url => context.config.url.join(&self.url)?,
            }
        })
    }
}
