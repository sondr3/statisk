use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::Path;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Config {
    url: Url,
    title: Option<String>,
    description: Option<String>,
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Config> {
        let content = read_to_string(path)?;
        toml::from_str(&content).context("Unable to parse config")
    }
}
