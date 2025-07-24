use std::path::{Path, PathBuf};

use mlua::{
    FromLua, IntoLua, Lua, LuaSerdeExt,
    prelude::{LuaResult, LuaValue},
};
use serde::{Deserialize, Serialize};

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
    pub fn with_root(&mut self, root: &Path) {
        self.out_dir = root.join(&self.out_dir);
        self.content = root.join(&self.content);
        self.templates = root.join(&self.templates);
        self.public = root.join(&self.public);
        self.css = root.join(&self.css);
        self.js = root.join(&self.js);
        self.root = root.to_path_buf();
    }
}
