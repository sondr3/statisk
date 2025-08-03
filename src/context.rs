use std::sync::Arc;

use anyhow::{Context as _, Result};
use dashmap::DashMap;
use serde::Serialize;

use crate::{
    BuildMode,
    asset::{Asset, PublicFile},
    content::Content,
    events::{Event, EventSender},
    ignorer::StatiskIgnore,
    lua::{
        output::{OutputKind, OutputMatch},
        statisk::LuaStatisk,
    },
    render::Renderer,
    templating::Templates,
};

pub const LIVERELOAD_JS: &str = include_str!("livereload.js");

#[derive(Debug)]
pub struct StatiskContext {
    pub statisk: LuaStatisk,
    renderer: Renderer,
    pub assets: Arc<DashMap<String, Asset>>,
    pub pages: Arc<DashMap<String, Content>>,
    pub templates: Templates,
    pub mode: BuildMode,
    events: EventSender,
}

impl StatiskContext {
    pub fn new(
        templates: Templates,
        statisk: LuaStatisk,
        renderer: Renderer,
        mode: BuildMode,
        events: EventSender,
    ) -> Self {
        Self {
            statisk,
            renderer,
            assets: Arc::new(DashMap::new()),
            pages: Arc::new(DashMap::new()),
            templates,
            mode,
            events,
        }
    }

    pub fn collect(&mut self) -> Result<()> {
        use crate::lua::output::OutputMatch::*;

        if self.mode.normal() {
            self.assets.insert(
                "livereload.js".to_string(),
                Asset {
                    source_name: "livereload.js".to_string(),
                    build_path: self.statisk.out_dir().join("livereload.js"),
                    content: LIVERELOAD_JS.to_string(),
                },
            );
        }

        for path in StatiskIgnore::walker(&self.statisk.root) {
            for output in &self.statisk.outputs {
                match output.match_kind(&path) {
                    Glob(OutputKind::File) => {}
                    Glob(OutputKind::PublicFile) | Watch(OutputKind::PublicFile) => {
                        output
                            .build(&path, &self.statisk.root, &self.statisk.out_dir())
                            .context("failed to build public file")?;
                    }
                    Glob(OutputKind::Template) => {}
                    Glob(OutputKind::Asset) => {}
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn build(&self) -> Result<()> {
        self.renderer.render_context(self)
    }

    pub fn update_asset(&self, key: String, asset: Asset) -> Result<()> {
        self.assets.insert(key, asset);
        self.renderer.write_assets(self)?;
        self.events.tx.send(Event::Reload).context("event failed")?;
        Ok(())
    }

    pub fn update_page(&self, key: String, page: Content) -> Result<()> {
        self.pages.insert(key, page);
        self.renderer.write_content(self)?;
        self.events.tx.send(Event::Reload).context("event failed")?;
        Ok(())
    }
}
