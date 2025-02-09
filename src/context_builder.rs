use std::path::Path;

use ahash::AHashMap;
use anyhow::Result;

use crate::{
    asset::{is_buildable_css_file, Asset, PublicFile},
    content::{Content, ContentType},
    context::Context,
    paths::{Paths, LIVERELOAD_JS},
    statisk_config::StatiskConfig,
    templating::{is_page, Templates},
    utils::{find_files, is_file},
    BuildMode,
};

pub struct ContextBuilder {
    pub assets: AHashMap<String, Asset>,
    pub pages: AHashMap<String, Content>,
    pub public_files: Vec<PublicFile>,
}

impl ContextBuilder {
    pub fn new(paths: &Paths, mode: BuildMode) -> Result<Self> {
        let pages = collect_content(paths)?;

        let mut assets = AHashMap::new();

        collect_css(paths, mode)?.into_iter().for_each(|a| {
            assets.insert(a.source_name.clone(), a);
        });
        collect_js(paths)?.into_iter().for_each(|a| {
            assets.insert(a.source_name.clone(), a);
        });

        if mode.normal() {
            assets.insert(
                "livereload.js".to_string(),
                Asset {
                    source_name: "livereload.js".to_string(),
                    build_path: paths.out.join(Path::new("livereload.js")),
                    content: LIVERELOAD_JS.to_string(),
                },
            );
        }

        let public_files = collect_public_files(paths);
        let mut pages: AHashMap<_, _> = pages.into_iter().map(|p| (p.filename(), p)).collect();
        pages.extend(
            collect_pages(&paths)?
                .into_iter()
                .map(|p| (p.filename(), p))
                .collect::<Vec<_>>(),
        );

        Ok(ContextBuilder {
            assets,
            pages,
            public_files,
        })
    }

    pub fn build(self, templates: Templates, config: StatiskConfig, mode: BuildMode) -> Context {
        Context::new(
            templates,
            config,
            self.assets,
            self.pages,
            self.public_files,
            mode,
        )
    }
}

fn collect_css(paths: &Paths, mode: BuildMode) -> Result<Vec<Asset>> {
    find_files(&paths.css, is_buildable_css_file)
        .map(|f| Asset::build_css(&f, mode))
        .collect()
}

fn collect_js(paths: &Paths) -> Result<Vec<Asset>> {
    find_files(&paths.js, is_file)
        .map(|f| Asset::from_path(&f))
        .collect()
}

pub fn collect_content(paths: &Paths) -> Result<Vec<Content>> {
    find_files(&paths.content, is_file)
        .map(|f| Content::from_path(&f, &paths.content, ContentType::Jotdown))
        .collect()
}

pub fn collect_pages(paths: &Paths) -> Result<Vec<Content>> {
    find_files(&paths.templates, is_file)
        .filter(|f| is_page(&f))
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
