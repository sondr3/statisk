use std::{collections::HashMap, fs::read_to_string, path::Path};

use anyhow::Result;
use mlua::{FromLua, Lua, Value, prelude::*};
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
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Author {
    pub name: String,
    pub description: Option<String>,
    pub contact: Option<String>,
}

impl FromLua for StatiskConfig {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for StatiskConfig {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
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
