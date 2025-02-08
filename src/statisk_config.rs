use std::{fs::read_to_string, path::Path};

use ahash::AHashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::build_mode::BuildMode;

#[derive(Debug, Deserialize, Serialize)]
pub struct StatiskConfig {
    pub url: Url,
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<Author>,
    #[serde(default)]
    pub extra: AHashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Author {
    pub name: String,
    pub description: Option<String>,
    pub contact: Option<String>,
}

impl StatiskConfig {
    pub fn from_path(path: &Path, mode: BuildMode) -> Result<StatiskConfig> {
        let content = read_to_string(path)?;
        let mut config: StatiskConfig = toml::from_str(&content)?;
        if mode.normal() {
            config.url = Url::parse("http://localhost:3000")?;
        }

        Ok(config)
    }
}
