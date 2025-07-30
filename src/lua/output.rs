use std::{fmt, fmt::Formatter, path::Path};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use mlua::{
    FromLua, Lua, UserData,
    prelude::{LuaError, LuaFunction, LuaResult, LuaUserDataMethods, LuaValue},
};

use crate::utils::new_copy_file;

#[derive(Debug, Copy, Clone)]
pub enum OutputKind {
    PublicFile,
    File,
    Template,
    Asset,
}

#[derive(Clone)]
pub struct Output {
    pub kind: OutputKind,
    pub glob_set: GlobSet,
    pub out_pattern: Option<String>,
    pub filter_fn: Option<LuaFunction>,
}

impl Default for Output {
    fn default() -> Self {
        Output {
            kind: OutputKind::File,
            glob_set: GlobSet::empty(),
            out_pattern: None,
            filter_fn: None,
        }
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LuaOutput")
            .field("kind", &self.kind)
            .field("glob", &self.glob_set.len())
            .field("out_pattern", &self.out_pattern)
            .field("filter_fn", &self.filter_fn.is_some())
            .finish()
    }
}

impl Output {
    pub fn is_match(&self, path: &Path) -> bool {
        self.glob_set.is_match(path)
    }

    pub fn build(&self, path: &Path, root: &Path, out_dir: &Path) -> LuaResult<()> {
        match (self.is_match(path), self.kind) {
            (true, OutputKind::Asset) => {}
            (true, OutputKind::PublicFile) => {
                new_copy_file(path.to_path_buf(), root, out_dir)?;
            }
            (true, OutputKind::File) => {}
            (true, OutputKind::Template) => {}
            _ => {}
        }

        Ok(())
    }
}

impl UserData for Output {}

impl FromLua for Output {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match &value {
            LuaValue::UserData(ud) => {
                if let Ok(builder) = ud.borrow::<OutputBuilder>() {
                    builder.clone().build()
                } else if let Ok(output) = ud.borrow::<Output>() {
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

#[derive(Clone)]
pub struct OutputBuilder {
    pub kind: OutputKind,
    pub glob_set: GlobSetBuilder,
    pub out_pattern: Option<String>,
    pub filter_fn: Option<LuaFunction>,
}

impl OutputBuilder {
    pub fn build(self) -> LuaResult<Output> {
        Ok(Output {
            kind: self.kind,
            glob_set: self.glob_set.build().context("Failed to build glob set")?,
            out_pattern: self.out_pattern,
            filter_fn: self.filter_fn,
        })
    }

    pub fn new(kind: OutputKind, glob: Glob) -> Result<Self> {
        Ok(Self {
            kind,
            glob_set: GlobSetBuilder::new().add(glob).to_owned(),
            out_pattern: None,
            filter_fn: None,
        })
    }
}

impl UserData for OutputBuilder {
    fn add_methods<'lua, M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("watch", |_lua, this, glob: String| {
            let glob = Glob::new(&glob).context("invalid glob pattern")?;
            this.glob_set.add(glob);
            Ok(this.clone())
        });

        methods.add_method_mut("filter", |_lua, this, filter_fn: LuaFunction| {
            this.filter_fn = Some(filter_fn);
            Ok(this.clone())
        });

        methods.add_method_mut("out", |_, this, pattern: String| {
            this.out_pattern = Some(pattern);
            Ok(this.clone())
        });
    }
}
