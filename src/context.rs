use std::{path::Path, sync::Arc};

use ahash::AHashMap;
use anyhow::{Context as _, Result};
use dashmap::DashMap;

use crate::{
    BuildMode,
    asset::{Asset, PublicFile, is_buildable_css_file, is_js},
    content::{Content, ContentType},
    events::{Event, EventSender},
    paths::{LIVERELOAD_JS, Paths},
    render::Renderer,
    statisk_config::StatiskConfig,
    templating::{Templates, is_page},
    utils::{find_files, is_file},
};

pub struct Context {
    pub config: StatiskConfig,
    renderer: Renderer,
    pub assets: Arc<DashMap<String, Asset>>,
    pub pages: Arc<DashMap<String, Content>>,
    pub public_files: Vec<PublicFile>,
    pub templates: Templates,
    pub mode: BuildMode,
    events: EventSender,
}

impl Context {
    pub fn new(
        templates: Templates,
        config: StatiskConfig,
        renderer: Renderer,
        mode: BuildMode,
        events: EventSender,
    ) -> Self {
        Self {
            config,
            renderer,
            assets: Arc::new(DashMap::new()),
            pages: Arc::new(DashMap::new()),
            public_files: Vec::new(),
            templates,
            mode,
            events,
        }
    }

    pub fn build(&self) -> Result<()> {
        self.renderer.render_context(self)
    }

    pub fn collect(&mut self, paths: &Paths) -> Result<()> {
        let pages = collect_content(paths)?;
        let mut pages: AHashMap<_, _> = pages.into_iter().map(|p| (p.filename(), p)).collect();
        pages.extend(
            collect_pages(paths)?
                .into_iter()
                .map(|p| (p.filename(), p))
                .collect::<Vec<_>>(),
        );

        for (key, page) in pages {
            self.pages.insert(key, page);
        }

        for asset in collect_css(paths, self.mode)? {
            self.assets.insert(asset.source_name.clone(), asset);
        }
        for asset in collect_js(paths, self.mode)? {
            self.assets.insert(asset.source_name.clone(), asset);
        }

        if self.mode.normal() {
            self.assets.insert(
                "livereload.js".to_string(),
                Asset {
                    source_name: "livereload.js".to_string(),
                    build_path: paths.out.join(Path::new("livereload.js")),
                    content: LIVERELOAD_JS.to_string(),
                },
            );
        }

        self.public_files.extend(collect_public_files(paths));

        Ok(())
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

fn collect_css(paths: &Paths, mode: BuildMode) -> Result<Vec<Asset>> {
    find_files(&paths.css, is_buildable_css_file)
        .map(|f| Asset::build_css(&f, mode))
        .collect()
}

fn collect_js(paths: &Paths, mode: BuildMode) -> Result<Vec<Asset>> {
    find_files(&paths.js, is_js)
        .map(|f| Asset::build_js(&f, mode))
        .collect()
}

pub fn collect_content(paths: &Paths) -> Result<Vec<Content>> {
    find_files(&paths.content, is_file)
        .map(|f| Content::from_path(&f, &paths.content, ContentType::Jotdown))
        .collect()
}

pub fn collect_pages(paths: &Paths) -> Result<Vec<Content>> {
    find_files(&paths.templates, is_file)
        .filter(|f| is_page(f))
        .map(|f| Content::from_path(&f, &paths.templates, ContentType::from_ext(&f)?))
        .collect()
}

fn collect_public_files(paths: &Paths) -> Vec<PublicFile> {
    find_files(&paths.public, is_file)
        .map(|f| PublicFile {
            path: f,
            prefix: paths.public.display().to_string(),
        })
        .collect()
}
