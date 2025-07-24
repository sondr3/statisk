use std::path::PathBuf;

use globset::GlobMatcher;
use mlua::{
    UserData,
    prelude::{LuaError, LuaResult, LuaUserDataMethods},
};

use crate::lua::output::{BuildOutput, Output};

#[derive(Debug, Clone)]
pub struct FileOutputBuilder {
    pub glob: GlobMatcher,
    pub output: Option<PathBuf>,
}

impl FileOutputBuilder {
    pub(crate) fn new(glob: GlobMatcher) -> Self {
        Self { glob, output: None }
    }
}

impl BuildOutput for FileOutputBuilder {
    fn build(self) -> LuaResult<Output> {
        let output = self
            .output
            .ok_or_else(|| LuaError::runtime("Template output must have a filter function"))?;

        Ok(Output::File {
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
