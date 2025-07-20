use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use mlua::{Table, prelude::*};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{build_mode::BuildMode, statisk_config::StatiskConfig};

#[derive(Debug)]
pub struct LuaStatisk {
    pub mode: BuildMode,
    pub config: StatiskConfig,
    pub paths: PathConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PathConfig {
    pub root: PathBuf,
    pub out_dir: PathBuf,
    pub content: PathBuf,
    pub templates: PathBuf,
    pub public: PathBuf,
    pub css: PathBuf,
    pub js: PathBuf,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            out_dir: PathBuf::from("_dist"),
            content: PathBuf::from("content"),
            templates: PathBuf::from("templates"),
            public: PathBuf::from("public"),
            css: PathBuf::from("css"),
            js: PathBuf::from("js"),
        }
    }
}

impl FromLua for PathConfig {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for PathConfig {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        lua.to_value(&self)
    }
}

impl PathConfig {
    pub fn with_root(&mut self, root: PathBuf) {
        self.out_dir = root.join(&self.out_dir);
        self.content = root.join(&self.content);
        self.templates = root.join(&self.templates);
        self.public = root.join(&self.public);
        self.css = root.join(&self.css);
        self.js = root.join(&self.js);
        self.root = root;
    }
}

impl FromLua for LuaStatisk {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            let mode: BuildMode = table.get("mode")?;
            let config: StatiskConfig = table.get("config")?;
            let paths: PathConfig = table.get("paths")?;

            let config = LuaStatisk {
                mode,
                config,
                paths,
            };
            Ok(config)
        } else {
            Err(LuaError::runtime("Statisk was not a config"))
        }
    }
}

impl IntoLua for LuaStatisk {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("mode", self.mode)?;
        table.set("config", self.config)?;
        table.set("paths", self.paths)?;

        Ok(LuaValue::Table(table))
    }
}

impl LuaStatisk {
    pub fn load(lua: &Lua, path: &Path) -> Result<Self> {
        anyhow::Context::context(lua.load(path).eval(), "Failed to load Lua script")
    }

    pub fn url(&self) -> Url {
        match self.mode {
            BuildMode::Normal => Url::from_str("http://localhost:3000").unwrap(),
            BuildMode::Optimized => self.config.url.clone(),
        }
    }
}

pub fn create_lua_context(mode: BuildMode, root: PathBuf) -> LuaResult<Lua> {
    let lua = Lua::new();
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    let statisk_table = lua.create_table()?;
    statisk_table.set("mode", lua.to_value(&mode)?)?;
    statisk_table.set(
        "setup",
        lua.create_function(move |_, config_table: LuaTable| {
            let config: StatiskConfig = config_table.get("config")?;
            let mut paths: PathConfig = config_table.get("paths")?;
            paths.with_root(root.clone());

            Ok(LuaStatisk {
                mode,
                config,
                paths,
            })
        })?,
    )?;
    loaded.set("statisk", statisk_table)?;

    Ok(lua)
}
