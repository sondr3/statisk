use globset::GlobMatcher;
use mlua::{UserData, prelude::LuaResult};

use crate::lua::output::{BuildOutput, Output};

#[derive(Debug, Clone)]
pub struct PublicFileOutputBuilder {
    pub glob: GlobMatcher,
}

impl PublicFileOutputBuilder {
    pub(crate) fn new(glob: GlobMatcher) -> Self {
        Self { glob }
    }
}

impl BuildOutput for PublicFileOutputBuilder {
    fn build(self) -> LuaResult<Output> {
        Ok(Output::PublicFile { glob: self.glob })
    }
}

impl UserData for PublicFileOutputBuilder {}
