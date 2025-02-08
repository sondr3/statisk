use std::path::{Path, PathBuf};

use anyhow::Result;
use jiff::civil::Date;
use jotdown::{Attributes, Container, Event, Render};
use minijinja::{context, value::Value};
use serde::Deserialize;

use crate::templating::{create_base_context, TemplatePath};
use crate::utils::unprefixed_parent;
use crate::{context::Context as SContext, utils::toml_date_option_deserializer, BuildMode};

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    #[serde(with = "toml_date_option_deserializer", default)]
    pub last_modified: Option<Date>,
    pub subtitle: Option<String>,
    pub description: String,
    pub slug: Option<String>,
    pub layout: Option<String>,
    #[serde(default)]
    pub special: bool,
}

#[derive(Debug)]
pub struct Content {
    pub source: PathBuf,
    pub out_path: PathBuf,
    pub dir: Option<String>,
    pub url: String,
    pub frontmatter: Frontmatter,
    pub content: String,
}

impl Content {
    pub fn from_path(path: &Path, root: &Path) -> Result<Self> {
        let file = std::fs::read_to_string(path)?;
        let stem = path.file_stem().unwrap().to_string_lossy();

        match file
            .split("+++")
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .collect::<Vec<_>>()[..]
        {
            [frontmatter, content] => {
                Content::from_file(path, root, &stem, frontmatter, Some(content))
            }
            [frontmatter] => Content::from_file(path, root, &stem, frontmatter, None),
            _ => todo!(),
        }
    }

    pub fn render(&self, mode: BuildMode, context: &SContext) -> Result<String> {
        let tmpl_context = self.context(context, mode)?;
        context
            .templates
            .render_template(&self.layout(), tmpl_context)
    }

    fn from_file(
        source: &Path,
        root: &Path,
        stem: &str,
        frontmatter: &str,
        content: Option<&str>,
    ) -> Result<Self> {
        let frontmatter: Frontmatter = toml::from_str(frontmatter)?;

        let path: PathBuf = match &frontmatter.slug {
            Some(slug) => [slug, "index.html"].into_iter().collect(),
            None => [stem, "index.html"].into_iter().collect(),
        };

        let url = frontmatter.slug.as_ref().map_or(stem, |s| s);
        let dir = unprefixed_parent(source, root);

        Ok(Content {
            source: source.to_path_buf(),
            url: format!("{url}/"),
            out_path: path,
            dir,
            content: content.unwrap_or_default().into(),
            frontmatter,
        })
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

    fn content(&self) -> Result<String> {
        let events = jotdown::Parser::new(&self.content).map(jotdown_event_mapper);
        let mut html = String::new();
        jotdown::html::Renderer::default().push(events, &mut html)?;
        Ok(html)
    }

    fn context(&self, context: &SContext, mode: BuildMode) -> Result<Value> {
        let base_context = create_base_context(mode, context);
        let content = self.content()?;

        Ok(context! {
            ..base_context,
            ..context! {
                title => self.frontmatter.title.clone(),
                subtitle => self.frontmatter.subtitle.clone(),
                description => self.frontmatter.description.clone(),
                content => content,
                canonical_url => context.config.url.join(&self.url)?,
            }
        })
    }
}

fn jotdown_event_mapper(event: jotdown::Event) -> jotdown::Event {
    match event {
        Event::Start(container, attrs) => jotdown_container_mapper(container, attrs).into(),
        _ => event,
    }
}

struct ContainerWrapper<'a>(Container<'a>, Attributes<'a>);

impl<'a> From<ContainerWrapper<'a>> for jotdown::Event<'a> {
    fn from(val: ContainerWrapper<'a>) -> Self {
        Event::Start(val.0, val.1)
    }
}

fn jotdown_container_mapper<'a>(
    container: Container<'a>,
    attrs: Attributes<'a>,
) -> ContainerWrapper<'a> {
    match container {
        Container::Heading {
            id,
            level,
            has_section,
        } => ContainerWrapper(
            Container::Heading {
                level,
                id: id.to_lowercase().into(),
                has_section,
            },
            attrs,
        ),
        Container::Section { id } => ContainerWrapper(
            Container::Section {
                id: id.to_lowercase().into(),
            },
            attrs,
        ),
        _ => ContainerWrapper(container, attrs),
    }
}
