use std::{
    fmt,
    fmt::Formatter,
    path::{Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};

use anyhow::{Context, Result};
use globset::{Glob, GlobMatcher};
use mlua::{RegistryKey, Table, UserData, prelude::*};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{build_mode::BuildMode, config::StatiskConfig, meta::StatiskMeta};

static ROOT_KEY: OnceLock<RegistryKey> = OnceLock::new();

#[derive(Debug)]
pub struct LuaStatisk {
    pub mode: BuildMode,
    pub root: PathBuf,
    pub meta: StatiskMeta,
    pub config: StatiskConfig,
    pub paths: PathConfig,
    pub outputs: Vec<LuaOutput>,
}

#[derive(Clone)]
pub enum LuaOutput {
    File {
        glob: GlobMatcher,
        output: PathBuf,
    },
    Template {
        glob: GlobMatcher,
        filter: LuaFunction,
        output_pattern: String,
    },
}

impl fmt::Debug for LuaOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LuaOutput::File { glob, output } => f
                .debug_struct("LuaOutput::File")
                .field("glob", &glob.glob().glob())
                .field("output", &output)
                .finish(),
            LuaOutput::Template {
                glob,
                output_pattern,
                ..
            } => f
                .debug_struct("LuaOutput::Template")
                .field("glob", &glob.glob().glob())
                .field("filter", &"lua_function")
                .field("output_pattern", &output_pattern)
                .finish(),
        }
    }
}

impl LuaOutput {
    pub fn is_match(&self, path: &Path) -> bool {
        match self {
            LuaOutput::File { glob, .. } => glob.is_match(path),
            LuaOutput::Template { glob, .. } => glob.is_match(path),
        }
    }
}

impl IntoLua for LuaOutput {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let ud = match self {
            LuaOutput::File { glob, output } => {
                let builder = FileOutputBuilder {
                    glob,
                    output: Some(output),
                };
                lua.create_userdata(builder)?
            }
            LuaOutput::Template {
                glob,
                filter,
                output_pattern,
            } => {
                let builder = TemplateOutputBuilder {
                    glob,
                    filter_fn: Some(filter),
                    output_pattern,
                };
                lua.create_userdata(builder)?
            }
        };
        Ok(LuaValue::UserData(ud))
    }
}

impl FromLua for LuaOutput {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match &value {
            LuaValue::UserData(ud) => {
                if let Ok(builder) = ud.borrow::<FileOutputBuilder>() {
                    builder.clone().build()
                } else if let Ok(builder) = ud.borrow::<TemplateOutputBuilder>() {
                    builder.clone().build()
                } else if let Ok(output) = ud.borrow::<LuaOutput>() {
                    Ok(output.clone())
                } else {
                    Err(LuaError::FromLuaConversionError {
                        from: "UserData",
                        to: "LuaOutput".to_string(),
                        message: Some(
                            "UserData is not a LuaOutputBuilder or LuaOutput".to_string(),
                        ),
                    })
                }
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaOutput".to_string(),
                message: Some(
                    "Expected a statisk.file() or statisk.template() builder".to_string(),
                ),
            }),
        }
    }
}

pub trait LuaBuildOutput {
    fn build(self) -> LuaResult<LuaOutput>;
}

#[derive(Debug, Clone)]
pub struct FileOutputBuilder {
    glob: GlobMatcher,
    output: Option<PathBuf>,
}

impl FileOutputBuilder {
    fn new(glob: GlobMatcher) -> Self {
        Self { glob, output: None }
    }
}

impl LuaBuildOutput for FileOutputBuilder {
    fn build(self) -> LuaResult<LuaOutput> {
        let output = self
            .output
            .ok_or_else(|| LuaError::runtime("Template output must have a filter function"))?;

        Ok(LuaOutput::File {
            glob: self.glob,
            output,
        })
    }
}

impl UserData for FileOutputBuilder {
    fn add_methods<'lua, M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("output", |_, this, output_path: String| {
            this.output = Some(PathBuf::from(output_path));
            Ok(this.clone())
        });
    }
}

#[derive(Debug, Clone)]
pub struct TemplateOutputBuilder {
    glob: GlobMatcher,
    filter_fn: Option<LuaFunction>,
    output_pattern: String,
}

impl TemplateOutputBuilder {
    fn new(glob: GlobMatcher) -> Self {
        Self {
            glob,
            filter_fn: None,
            output_pattern: "{slug}.html".to_string(),
        }
    }
}

impl LuaBuildOutput for TemplateOutputBuilder {
    fn build(self) -> LuaResult<LuaOutput> {
        let filter_fn = self
            .filter_fn
            .ok_or_else(|| LuaError::runtime("Template output must have a filter function"))?;

        Ok(LuaOutput::Template {
            glob: self.glob,
            filter: filter_fn,
            output_pattern: self.output_pattern,
        })
    }
}

impl UserData for TemplateOutputBuilder {
    fn add_methods<'lua, M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("filter", |_lua, this, filter_fn: LuaFunction| {
            this.filter_fn = Some(filter_fn);
            Ok(this.clone())
        });

        methods.add_method_mut("pattern", |_, this, pattern: String| {
            this.output_pattern = pattern;
            Ok(this.clone())
        });
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
    pub fn load(lua: &Lua, path: &Path) -> Result<Self> {
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
