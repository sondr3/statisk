use std::fmt::Display;

use clap::ValueEnum;
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, ValueEnum, Serialize, Deserialize)]
pub enum BuildMode {
    Optimized,
    Normal,
}

impl Display for BuildMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildMode::Optimized => write!(f, "optimized"),
            BuildMode::Normal => write!(f, "normal"),
        }
    }
}

impl BuildMode {
    #[must_use]
    pub const fn optimize(self) -> bool {
        matches!(self, Self::Optimized)
    }

    #[must_use]
    pub const fn normal(self) -> bool {
        matches!(self, Self::Normal)
    }
}

impl IntoLua for BuildMode {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for BuildMode {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
