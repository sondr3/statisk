use anyhow::{Context, Result};
use jiff::civil::Date;
use minijinja::{context, Value};
use serde::{Deserialize, Serialize};

use crate::{sitemap::ChangeFreq, utils::toml_date_jiff_serde};

#[derive(Debug, Deserialize, Serialize)]
pub struct Frontmatter {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    pub slug: Option<String>,
    pub layout: Option<String>,
    pub change_freq: Option<ChangeFreq>,
    #[serde(with = "toml_date_jiff_serde", default)]
    pub last_modified: Option<Date>,
    #[serde(with = "toml_date_jiff_serde", default)]
    pub created: Option<Date>,
}

impl Frontmatter {
    pub fn deserialize(input: &str) -> Result<Self> {
        toml::from_str(input).context("Could not parse frontmatter")
    }

    pub fn url(&self, stem: &str) -> String {
        self.slug.as_ref().map_or(stem, |s| s).to_string()
    }

    pub fn to_context(&self) -> Value {
        context! {
            title => &self.title,
            subtitle => &self.subtitle,
            description => &self.description,
            slug => &self.slug,
            last_modified => &self.last_modified,
            created => &self.created
        }
    }
}
