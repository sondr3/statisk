use ahash::AHashMap;

use crate::{
    asset::{Asset, PublicFile},
    content::Content,
    statisk_config::StatiskConfig,
    templating::Templates,
    BuildMode,
};

pub struct Context {
    pub config: StatiskConfig,
    pub assets: AHashMap<String, Asset>,
    pub pages: AHashMap<String, Content>,
    pub public_files: Vec<PublicFile>,
    pub templates: Templates,
    pub mode: BuildMode,
}

impl Context {
    pub fn new(
        templates: Templates,
        config: StatiskConfig,
        assets: AHashMap<String, Asset>,
        pages: AHashMap<String, Content>,
        public_files: Vec<PublicFile>,
        mode: BuildMode,
    ) -> Self {
        Self {
            config,
            assets,
            pages,
            templates,
            public_files,
            mode,
        }
    }
}
