use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use minijinja::{context, value::Value};

use crate::{
    context::Context as SContext,
    frontmatter::Frontmatter,
    jotdown::render_jotdown,
    templating::{create_base_context, TemplatePath},
    utils::{split_frontmatter, unprefixed_parent},
    BuildMode,
};

#[derive(Debug)]
pub enum ContentType {
    Template,
    Jotdown,
}

#[derive(Debug)]
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
            split_frontmatter(file).ok_or(anyhow!("Could not find content or frontmatter"))?;

        let Some(frontmatter) = frontmatter else {
            bail!("Missing frontmatter in content");
        };
        let frontmatter = Frontmatter::deserialize(&frontmatter)?;

        let dir = unprefixed_parent(&path, root);
        let out_path: PathBuf = match kind {
            ContentType::Template => match (&dir, &frontmatter.slug) {
                (None, None) => PathBuf::from("index.html"),
                (None, Some(slug)) => [slug, "index.html"].into_iter().collect(),
                (Some(dir), Some(slug)) => [&dir, &slug, "index.html"].into_iter().collect(),
                (Some(dir), None) => [&dir, "index.html"].into_iter().collect(),
            },
            ContentType::Jotdown => match &frontmatter.slug {
                Some(slug) => [slug, "index.html"].into_iter().collect(),
                None => [stem, "index.html"].into_iter().collect(),
            },
        };

        let url = frontmatter.url(stem);

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
            ContentType::Template => self.render_template(mode, context),
            ContentType::Jotdown => self.render_jotdown(mode, context),
        }
    }

    pub fn filename(&self) -> String {
        self.source.file_stem().map_or_else(
            || panic!("No filename found"),
            |name| name.to_string_lossy().to_string(),
        )
    }

    fn layout(&self) -> TemplatePath {
        match &self.frontmatter.layout {
            Some(layout) => TemplatePath(None, layout.to_string()),
            None => TemplatePath(self.dir.clone(), "page".to_string()),
        }
    }

    fn render_jotdown(&self, mode: BuildMode, context: &SContext) -> Result<String> {
        let tmpl_context = self.context(context, mode)?;
        context
            .templates
            .render_template(&self.layout(), tmpl_context)
    }

    fn render_template(&self, mode: BuildMode, app_context: &SContext) -> Result<String> {
        let context = self.context(app_context, mode)?;
        let env = app_context.templates.environment.acquire_env()?;
        let template = env.template_from_str(&self.content)?;
        template.render(context).context("Could not render")
    }

    fn context(&self, context: &SContext, mode: BuildMode) -> Result<Value> {
        let base_context = create_base_context(mode, context);
        let content = render_jotdown(&self.content)?;
        let frontmatter_context = self.frontmatter.to_context();

        Ok(context! {
            ..base_context,
            ..frontmatter_context,
            ..context! {
                content => content,
                canonical_url => context.config.url.join(&self.url)?,
            }
        })
    }
}
