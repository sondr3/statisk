use std::{
    fmt,
    fmt::Formatter,
    path::{Path, PathBuf},
};

use globset::GlobMatcher;
use mlua::{
    FromLua, IntoLua, Lua,
    prelude::{LuaError, LuaFunction, LuaResult, LuaValue},
};

use crate::lua::{file_builder::FileOutputBuilder, template_builder::TemplateOutputBuilder};

pub trait LuaBuildOutput {
    fn build(self) -> LuaResult<LuaOutput>;
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
