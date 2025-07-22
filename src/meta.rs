use std::collections::HashMap;

use mlua::{FromLua, Lua, Value, prelude::*};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct StatiskMeta {
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

impl FromLua for StatiskMeta {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for StatiskMeta {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}
