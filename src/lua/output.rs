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

use crate::{
    lua::{
        file_builder::FileOutputBuilder, public_file_builder::PublicFileOutputBuilder,
        template_builder::TemplateOutputBuilder,
    },
    utils::new_copy_file,
};

pub trait BuildOutput {
    fn build(self) -> LuaResult<Output>;
}

#[derive(Clone)]
pub enum Output {
    PublicFile {
        glob: GlobMatcher,
    },
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

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Output::PublicFile { glob } => f
                .debug_struct("LuaOutput::PublicFile")
                .field("glob", &glob.glob().glob())
                .finish(),
            Output::File { glob, output } => f
                .debug_struct("LuaOutput::File")
                .field("glob", &glob.glob().glob())
                .field("output", &output)
                .finish(),
            Output::Template {
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

impl Output {
    pub fn glob_pattern(&self) -> &str {
        match self {
            Output::PublicFile { glob } => glob.glob().glob(),
            Output::File { glob, .. } => glob.glob().glob(),
            Output::Template { glob, .. } => glob.glob().glob(),
        }
    }

    pub fn is_match(&self, path: &Path) -> bool {
        match self {
            Output::PublicFile { glob } => glob.is_match(path),
            Output::File { glob, .. } => glob.is_match(path),
            Output::Template { glob, .. } => glob.is_match(path),
        }
    }

    pub fn build(&self, path: &Path, root: &Path, out_dir: &Path) -> LuaResult<()> {
        match (self.is_match(path), self) {
            (true, Output::PublicFile { .. }) => {
                new_copy_file(path.to_path_buf(), root, out_dir)?;
            }
            (true, Output::File { .. }) => {}
            (true, Output::Template { .. }) => {}
            _ => {}
        }

        Ok(())
    }
}

impl IntoLua for Output {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let ud = match self {
            Output::PublicFile { glob } => {
                let builder = PublicFileOutputBuilder { glob };
                lua.create_userdata(builder)?
            }
            Output::File { glob, output } => {
                let builder = FileOutputBuilder {
                    glob,
                    output: Some(output),
                };
                lua.create_userdata(builder)?
            }
            Output::Template {
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

impl FromLua for Output {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match &value {
            LuaValue::UserData(ud) => {
                if let Ok(builder) = ud.borrow::<FileOutputBuilder>() {
                    builder.clone().build()
                } else if let Ok(builder) = ud.borrow::<TemplateOutputBuilder>() {
                    builder.clone().build()
                } else if let Ok(builder) = ud.borrow::<PublicFileOutputBuilder>() {
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
