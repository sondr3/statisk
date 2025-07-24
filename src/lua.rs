mod file_builder;
mod output;
pub mod path_config;
pub mod statisk;
mod template_builder;

use std::{path::PathBuf, sync::OnceLock};

use anyhow::Context;
use globset::Glob;
use mlua::{RegistryKey, Table, prelude::*};

use crate::{
    build_mode::BuildMode,
    lua::{
        file_builder::FileOutputBuilder, statisk::LuaStatisk,
        template_builder::TemplateOutputBuilder,
    },
};

pub static ROOT_KEY: OnceLock<RegistryKey> = OnceLock::new();

pub fn create_lua_context(mode: BuildMode, root: PathBuf) -> LuaResult<Lua> {
    let lua = Lua::new();
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    let root_key = lua.create_registry_value(root)?;
    ROOT_KEY.set(root_key).expect("Failed to set ROOT_KEY");
    let statisk_table = lua.create_table()?;
    statisk_table.set("mode", lua.to_value(&mode)?)?;

    statisk_table.set(
        "file",
        lua.create_function(|_, template: String| {
            let glob = Glob::new(&template)
                .context("invalid regex")?
                .compile_matcher();
            Ok(FileOutputBuilder::new(glob))
        })?,
    )?;

    statisk_table.set(
        "template",
        lua.create_function(|_, template: String| {
            let glob = Glob::new(&template)
                .context("invalid regex")?
                .compile_matcher();
            Ok(TemplateOutputBuilder::new(glob))
        })?,
    )?;

    statisk_table.set(
        "setup",
        lua.create_function(move |lua, config_table: LuaTable| {
            LuaStatisk::from_lua(LuaValue::Table(config_table), &lua)
        })?,
    )?;

    loaded.set("statisk", statisk_table)?;

    Ok(lua)
}
