use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use mlua::{
    FromLua, IntoLua, Lua, Table,
    prelude::{LuaError, LuaResult, LuaValue},
};
use url::Url;

use crate::{
    build_mode::BuildMode,
    config::StatiskConfig,
    lua::{ROOT_KEY, output::LuaOutput, path_config::PathConfig},
    meta::StatiskMeta,
};

#[derive(Debug)]
pub struct LuaStatisk {
    pub mode: BuildMode,
    pub root: PathBuf,
    pub meta: StatiskMeta,
    pub config: StatiskConfig,
    pub paths: PathConfig,
    pub outputs: Vec<LuaOutput>,
}

impl FromLua for LuaStatisk {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            let globals = lua.globals();
            let package: Table = globals.get("package")?;
            let loaded: Table = package.get("loaded")?;
            let statisk_table: Table = loaded.get("statisk")?;
            let mode: BuildMode = statisk_table.get("mode")?;

            let root: PathBuf = lua.registry_value(ROOT_KEY.get().unwrap())?;
            let meta: StatiskMeta = table.get("meta")?;
            let config: StatiskConfig = table.get("config").unwrap_or_default();
            let mut paths: PathConfig = table.get("paths")?;
            paths.with_root(&root);

            let outputs: Vec<LuaOutput> = table.get("outputs")?;

            let config = LuaStatisk {
                mode,
                root,
                meta,
                config,
                paths,
                outputs,
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
        table.set("root", self.root)?;
        table.set("meta", self.meta)?;
        table.set("config", self.config)?;
        table.set("paths", self.paths)?;
        table.set("outputs", self.outputs)?;

        Ok(LuaValue::Table(table))
    }
}

impl LuaStatisk {
    pub fn load(lua: &Lua, path: &Path) -> anyhow::Result<Self> {
        anyhow::Context::context(lua.load(path).eval(), "Failed to load Lua script")
    }

    pub fn url(&self) -> Url {
        match self.mode {
            BuildMode::Normal => Url::from_str("http://localhost:3000").unwrap(),
            BuildMode::Optimized => self.meta.url.clone(),
        }
    }

    pub fn template_root(&self) -> PathBuf {
        self.root.join(&self.config.template_root)
    }

    pub fn public_files(&self) -> PathBuf {
        self.root.join(&self.config.public_files)
    }

    pub fn out_dir(&self) -> PathBuf {
        self.root.join(&self.config.out_dir)
    }
}
