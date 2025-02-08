use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use ahash::AHashMap;
use anyhow::{anyhow, bail, Context, Result};
use jiff::civil::Date;
use minijinja::{context, path_loader, Environment, State, Value};
use minijinja_autoreload::AutoReloader;
use minijinja_contrib::add_to_environment;
use serde::{Deserialize, Serialize};

use crate::{
    build_mode::BuildMode,
    context::Context as SContext,
    utils::{
        filename, find_files, is_file, split_frontmatter, toml_date_jiff_serde, unprefixed_parent,
    },
};

pub fn is_page(path: &Path) -> bool {
    !is_template(path) && !is_partial(path)
}

pub fn is_partial(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|f| f.to_str().is_some_and(|p| p.starts_with("_")))
}

pub fn is_template(path: &Path) -> bool {
    path.file_stem().is_some_and(|f| {
        f.to_str()
            .is_some_and(|p| p.starts_with("[") && p.ends_with("]"))
    })
}

fn get_asset(state: &State, name: &str) -> Option<Value> {
    let context = &state.lookup("assets")?;
    let asset = context.get_attr(name).ok()?;
    if asset.is_undefined() {
        return None;
    }

    let path = asset.get_attr("build_path").ok()?;
    let filename = filename(path.to_string());
    Some(filename.into())
}

#[derive(Debug)]
pub struct Template {
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PageFrontmatter {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    pub slug: Option<String>,
    pub layout: Option<String>,
    #[serde(with = "toml_date_jiff_serde", default)]
    pub last_modified: Option<Date>,
    #[serde(with = "toml_date_jiff_serde", default)]
    pub created: Option<Date>,
}

#[derive(Debug)]
pub struct TemplatePage {
    pub frontmatter: Option<PageFrontmatter>,
    pub content: String,
}

impl TemplatePage {
    pub fn new(content: String) -> Result<Self> {
        match split_frontmatter(content) {
            Some((Some(frontmatter), content)) => {
                let frontmatter =
                    toml::from_str(&frontmatter).context("Could not parse frontmatter")?;
                Ok(TemplatePage {
                    frontmatter: Some(frontmatter),
                    content,
                })
            }
            Some((None, content)) => Ok(TemplatePage {
                frontmatter: None,
                content,
            }),
            None => bail!("No content or frontmatter found for page"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TemplatePath(pub Option<String>, pub String);

impl TemplatePath {
    pub fn as_path_buf(&self) -> PathBuf {
        let (dir, file) = (&self.0, &self.1);
        if let Some(dir) = dir {
            return [dir, file].iter().collect();
        }

        PathBuf::from(file)
    }
}

pub struct Templates {
    pub environment: AutoReloader,
    pub templates: AHashMap<TemplatePath, Template>,
    pub pages: AHashMap<TemplatePath, TemplatePage>,
}

pub fn create_base_context(mode: BuildMode, context: &SContext) -> Value {
    context! {
        mode => mode,
        is_dev => mode.normal(),
        assets => context.assets,
        config => context.config,
    }
}

impl Templates {
    pub fn new(root: PathBuf) -> Result<Self> {
        let template_path = root.clone();
        let env = AutoReloader::new(move |notifier| {
            let mut env = Environment::new();
            env.set_loader(path_loader(&template_path));
            add_to_environment(&mut env);
            env.add_function("get_asset", get_asset);

            notifier.set_fast_reload(true);

            notifier.watch_path(&template_path, true);
            Ok(env)
        });

        let mut templates = Templates {
            environment: env,
            templates: AHashMap::new(),
            pages: AHashMap::new(),
        };

        for file in find_files(&root, is_file) {
            templates.add_template(file, &root)?;
        }

        Ok(templates)
    }

    pub fn add_template(&mut self, path: PathBuf, root: &Path) -> Result<()> {
        let name = path
            .file_stem()
            .ok_or(anyhow!("No file name"))?
            .to_string_lossy()
            .replace("[", "")
            .replace("]", "");

        let dir = unprefixed_parent(&path, root);

        let content = read_to_string(&path).context("could not read file")?;
        let tmpl_path = TemplatePath(dir, name);
        if is_page(&path) {
            let page = TemplatePage::new(content)?;
            self.pages.insert(tmpl_path, page);
        } else if is_template(&path) {
            let template = Template { content };
            self.templates.insert(tmpl_path, template);
        }

        Ok(())
    }

    pub fn render_template(&self, path: &TemplatePath, context: Value) -> Result<String> {
        let template = self
            .templates
            .get(path)
            .ok_or(anyhow!("Could not find template"))?;

        let env = self.environment.acquire_env()?;
        let template = env.template_from_str(&template.content)?;
        template.render(context).context("Could not render")
    }

    pub fn render_template_page(&self, path: &TemplatePath, context: Value) -> Result<String> {
        let template = self
            .pages
            .get(path)
            .ok_or(anyhow!("Could not find template"))?;

        let page_context = match &template.frontmatter {
            Some(frontmatter) => {
                context! {
                    title => frontmatter.title.clone(),
                    subtitle => frontmatter.subtitle.clone(),
                    description => frontmatter.description.clone(),
                }
            }
            None => context! {},
        };

        let context = context! {
            ..context,
            ..page_context
        };

        let env = self.environment.acquire_env()?;
        let template = env.template_from_str(&template.content)?;
        template.render(context).context("Could not render")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::templating::{is_page, is_template};

    #[test]
    fn test_is_page() {
        let pages = vec!["index.html", "about.jinja", "404.html"];
        for page in pages {
            assert!(is_page(&PathBuf::from(page)));
        }
    }

    #[test]
    fn test_is_template() {
        let pages = vec!["pages/[slug].html", "posts/[post].jinja"];
        for page in pages {
            assert!(is_template(&PathBuf::from(page)));
        }
    }
}
