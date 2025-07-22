use std::path::PathBuf;

use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StatiskConfig {
    pub out_dir: PathBuf,
    pub template_root: PathBuf,
    pub public_files: PathBuf,
}

impl Default for StatiskConfig {
    fn default() -> Self {
        StatiskConfig {
            out_dir: PathBuf::from("_dist"),
            template_root: PathBuf::from("templates"),
            public_files: PathBuf::from("public"),
        }
    }
}

impl IntoLua for StatiskConfig {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for StatiskConfig {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
