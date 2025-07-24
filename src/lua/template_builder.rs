use globset::GlobMatcher;
use mlua::{
    UserData,
    prelude::{LuaError, LuaFunction, LuaResult, LuaUserDataMethods},
};

use crate::lua::output::{LuaBuildOutput, LuaOutput};

#[derive(Debug, Clone)]
pub struct TemplateOutputBuilder {
    pub glob: GlobMatcher,
    pub filter_fn: Option<LuaFunction>,
    pub output_pattern: String,
}

impl TemplateOutputBuilder {
    pub(crate) fn new(glob: GlobMatcher) -> Self {
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
