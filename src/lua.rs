use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use mlua::{RegistryKey, Table, UserData, prelude::*};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{build_mode::BuildMode, statisk_config::StatiskConfig};

#[derive(Debug)]
pub struct LuaStatisk {
    pub mode: BuildMode,
    pub config: StatiskConfig,
    pub paths: PathConfig,
    pub outputs: Vec<LuaOutput>,
}

#[derive(Debug, Clone)]
pub enum LuaOutput {
    File {
        template: PathBuf,
        output: PathBuf,
    },
    Template {
        template: PathBuf,
        filter: LuaFunction,
        output_pattern: String,
    },
}

impl FromLua for LuaOutput {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            if let Ok(output) = ud.borrow::<LuaOutput>() {
                return Ok(output.clone());
            }
        }
        Err(LuaError::runtime("Expected LuaOutput"))
    }
}

impl UserData for LuaOutput {}

#[derive(Debug, Clone)]
pub struct FileOutputBuilder {
    template_path: PathBuf,
    output: Option<PathBuf>,
}

impl FileOutputBuilder {
    fn new(template_path: PathBuf) -> Self {
        Self {
            template_path,
            output: None,
        }
    }

    fn build(self) -> LuaResult<LuaOutput> {
        let output = self
            .output
            .ok_or_else(|| LuaError::runtime("Template output must have a filter function"))?;

        Ok(LuaOutput::File {
            template: self.template_path,
            output,
        })
    }
}

// Builder for template outputs - more complex with chaining methods
#[derive(Debug, Clone)]
pub struct TemplateOutputBuilder {
    template_path: PathBuf,
    filter_fn: Option<LuaFunction>,
    output_pattern: String,
}

impl TemplateOutputBuilder {
    fn new(template_path: PathBuf) -> Self {
        Self {
            template_path,
            filter_fn: None,
            output_pattern: "{slug}.html".to_string(),
        }
    }

    fn build(self) -> LuaResult<LuaOutput> {
        let filter_fn = self
            .filter_fn
            .ok_or_else(|| LuaError::runtime("Template output must have a filter function"))?;

        Ok(LuaOutput::Template {
            template: self.template_path,
            filter: filter_fn,
            output_pattern: self.output_pattern,
        })
    }
}

impl LuaUserData for FileOutputBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("output", |_lua, this, output_path: PathBuf| {
            this.output = Some(output_path);
            Ok(this.clone())
        });

        methods.add_function("build", |_lua, this: FileOutputBuilder| {
            this.clone().build()
        });
    }
}

impl FromLua for FileOutputBuilder {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            if let Ok(builder) = ud.borrow::<FileOutputBuilder>() {
                return Ok(builder.clone());
            }
        }
        Err(LuaError::runtime("Expected TemplateOutputBuilder"))
    }
}

impl LuaUserData for TemplateOutputBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("filter", |_lua, this, filter_fn: LuaFunction| {
            this.filter_fn = Some(filter_fn);
            Ok(this.clone())
        });

        methods.add_method_mut("pattern", |_lua, this, pattern: String| {
            this.output_pattern = pattern;
            Ok(this.clone())
        });

        methods.add_function("build", |_lua, this: TemplateOutputBuilder| {
            this.clone().build()
        });
    }
}

impl FromLua for TemplateOutputBuilder {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            if let Ok(builder) = ud.borrow::<TemplateOutputBuilder>() {
                return Ok(builder.clone());
            }
        }
        Err(LuaError::runtime("Expected TemplateOutputBuilder"))
    }
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
            let outputs: Vec<LuaOutput> = table.get("outputs")?;

            let config = LuaStatisk {
                mode,
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
        table.set("config", self.config)?;
        table.set("paths", self.paths)?;
        table.set("outputs", self.outputs)?;

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

    let root_key: &'static RegistryKey = Box::leak(Box::new(lua.create_registry_value(root)?));
    let statisk_table = lua.create_table()?;
    statisk_table.set("mode", lua.to_value(&mode)?)?;

    statisk_table.set(
        "file",
        lua.create_function(|lua, template: PathBuf| {
            let root: PathBuf = lua.registry_value(root_key)?;
            let file_path = root.join(template);
            let builder = FileOutputBuilder::new(file_path);
            lua.create_userdata(builder)
        })?,
    )?;

    statisk_table.set(
        "template",
        lua.create_function(|lua, template: PathBuf| {
            let root: PathBuf = lua.registry_value(root_key)?;
            let file_path = root.join(template);
            let builder = TemplateOutputBuilder::new(file_path);
            lua.create_userdata(builder)
        })?,
    )?;

    statisk_table.set(
        "setup",
        lua.create_function(move |lua, config_table: LuaTable| {
            let root: PathBuf = lua.registry_value(root_key)?;
            let config: StatiskConfig = config_table.get("config")?;
            let outputs: Vec<LuaOutput> = config_table.get("outputs")?;

            let mut paths: PathConfig = config_table.get("paths")?;
            paths.with_root(root.clone());

            Ok(LuaStatisk {
                mode,
                config,
                paths,
                outputs,
            })
        })?,
    )?;

    loaded.set("statisk", statisk_table)?;

    Ok(lua)
}
