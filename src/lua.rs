use std::{fs::read_to_string, path::Path};

use mlua::{Table, prelude::*};

use crate::statisk_config::StatiskConfig;

#[derive(Debug)]
pub struct LuaStatisk {
    pub config: StatiskConfig,
}

impl FromLua for LuaStatisk {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            let config: StatiskConfig = table.get("config")?;
            Ok(LuaStatisk { config })
        } else {
            Err(LuaError::runtime("Statisk was not a config"))
        }
    }
}

impl IntoLua for LuaStatisk {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("config", self.config)?;
        Ok(LuaValue::Table(table))
    }
}

pub fn lua_context(path: &Path) -> LuaResult<Lua> {
    let lua = Lua::new();
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    let statisk_table = lua.create_table()?;
    StatiskConfig::create_context(&statisk_table, &lua)?;

    loaded.set("statisk", statisk_table)?;

    let file = read_to_string(path)?;
    let config = lua.load(file).eval::<LuaStatisk>()?;
    dbg!(config);

    Ok(lua)
}
