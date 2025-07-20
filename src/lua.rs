use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use mlua::{Table, prelude::*};
use serde::{Deserialize, Serialize};

use crate::statisk_config::StatiskConfig;

#[derive(Debug)]
pub struct LuaStatisk {
    pub config: StatiskConfig,
    pub paths: PathConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PathConfig {
    pub content: PathBuf,
    pub templates: PathBuf,
    pub public: PathBuf,
    pub css: PathBuf,
    pub js: PathBuf,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
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

impl FromLua for LuaStatisk {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            let config: StatiskConfig = table.get("config")?;
            let paths: PathConfig = table.get("paths")?;

            let config = LuaStatisk { config, paths };
            Ok(config)
        } else {
            Err(LuaError::runtime("Statisk was not a config"))
        }
    }
}

impl IntoLua for LuaStatisk {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("config", self.config)?;
        table.set("paths", self.paths)?;

        Ok(LuaValue::Table(table))
    }
}

pub fn lua_context(path: &Path) -> LuaResult<Lua> {
    let lua = Lua::new();
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    let statisk_table = lua.create_table()?;
    statisk_table.set(
        "setup",
        lua.create_function(|_, statisk: LuaStatisk| Ok(statisk))?,
    )?;
    loaded.set("statisk", statisk_table)?;

    let file = read_to_string(path)?;
    let config = lua.load(file).eval::<LuaStatisk>()?;
    dbg!(config);

    Ok(lua)
}
