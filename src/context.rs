use ahash::AHashMap;
use minijinja::{path_loader, Environment, State, Value};
use minijinja_autoreload::AutoReloader;
use minijinja_contrib::add_to_environment;

use crate::{
    asset::{Asset, PublicFile},
    content::Content,
    paths::Paths,
    statisk_config::StatiskConfig,
    utils::filename,
    BuildMode,
};

pub struct Context {
    pub config: StatiskConfig,
    pub assets: AHashMap<String, Asset>,
    pub pages: AHashMap<String, Content>,
    pub templates: AutoReloader,
    pub public_files: Vec<PublicFile>,
    pub mode: BuildMode,
}

fn get_asset(state: &State, name: &str) -> Option<Value> {
    let context = &state.lookup("assets")?;
    let asset = context.get_attr(name).ok()?;
    if asset.is_undefined() {
        return None;
    }

    let path = asset.get_attr("build_path").ok()?;
    let filename = filename(path.to_string());
    Some(filename.into())
}

impl Context {
    pub fn new(
        paths: &Paths,
        config: StatiskConfig,
        assets: AHashMap<String, Asset>,
        pages: AHashMap<String, Content>,
        public_files: Vec<PublicFile>,
        mode: BuildMode,
    ) -> Self {
        let template_path = paths.templates.clone();
        let env = AutoReloader::new(move |notifier| {
            let mut env = Environment::new();
            add_to_environment(&mut env);
            env.set_loader(path_loader(&template_path));
            env.add_function("get_asset", get_asset);

            notifier.set_fast_reload(true);

            notifier.watch_path(&template_path, true);
            Ok(env)
        });

        Self {
            config,
            assets,
            pages,
            templates: env,
            public_files,
            mode,
        }
    }

    pub fn _update_asset(&mut self, path: impl Into<String>, asset: Asset) {
        self.assets.insert(path.into(), asset);
    }

    pub fn _update_page(&mut self, path: impl Into<String>, page: Content) {
        self.pages.insert(path.into(), page);
    }
}
