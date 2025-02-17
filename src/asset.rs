use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::Result;
use lightningcss::{
    bundler::{Bundler, FileProvider},
    printer::PrinterOptions,
    stylesheet::ParserOptions,
};
use oxc_span::SourceType;
use serde::Serialize;
use walkdir::DirEntry;

use crate::{
    build_mode::BuildMode,
    minify,
    utils::{digest_filename, filename},
};

#[derive(Debug)]
pub struct PublicFile {
    pub path: PathBuf,
    pub prefix: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Asset {
    pub source_name: String,
    pub build_path: PathBuf,
    pub content: String,
}

impl Asset {
    pub fn _from_path(path: &Path) -> Result<Self> {
        let content = read_to_string(path)?;
        let filename = filename(path);

        Ok(Self {
            source_name: filename.clone(),
            build_path: path.to_owned(),
            content,
        })
    }

    pub fn build_css(path: &Path, mode: BuildMode) -> Result<Self> {
        let fs = Box::leak(Box::new(FileProvider::new()));
        let mut bundler = Bundler::new(fs, None, ParserOptions::default());
        let stylesheet = bundler.bundle(path)?;
        let css = stylesheet.to_css(PrinterOptions::default())?;
        let source_name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap();

        Ok(match mode {
            BuildMode::Optimized => Self {
                source_name,
                build_path: digest_filename(path, &css.code),
                content: minify::css(&css.code.clone())?,
            },
            BuildMode::Normal => Self {
                source_name,
                build_path: path.to_owned(),
                content: css.code,
            },
        })
    }

    pub fn build_js(path: &Path, mode: BuildMode) -> Result<Self> {
        let source = read_to_string(path)?;
        let source_name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap();

        let ext = SourceType::from_path(path)?;

        Ok(match mode {
            BuildMode::Optimized => Self {
                source_name,
                build_path: digest_filename(path, &source),
                content: minify::js(&source, Some(ext)),
            },
            BuildMode::Normal => Self {
                source_name,
                build_path: path.to_owned(),
                content: source,
            },
        })
    }
}

pub fn is_js(entry: &DirEntry) -> bool {
    entry
        .path()
        .extension()
        .is_some_and(|e| ["js", "mjs", "cjs"].contains(&e.to_string_lossy().as_ref()))
}

pub fn is_buildable_css_file(entry: &DirEntry) -> bool {
    !entry
        .file_name()
        .to_str()
        .is_some_and(|f| f.starts_with('_'))
        && entry.path().extension().is_some_and(|p| p == "css")
}
