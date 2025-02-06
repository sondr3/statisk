use std::path::{Path, PathBuf};

use anyhow::Result;
use lightningcss::{
    bundler::{Bundler, FileProvider},
    printer::PrinterOptions,
    stylesheet::ParserOptions,
};
use walkdir::DirEntry;

use crate::{
    minify,
    utils::{digest_filename, filename},
    Mode,
};

#[derive(Debug)]
pub struct PublicFile {
    pub path: PathBuf,
    pub prefix: String,
}

#[derive(Debug)]
pub struct Asset {
    pub source_name: String,
    pub build_path: PathBuf,
    pub content: String,
}

impl Asset {
    pub fn from_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let filename = filename(path);

        Ok(Self {
            source_name: filename.clone(),
            build_path: path.to_owned(),
            content,
        })
    }

    pub fn build_css(path: &Path, mode: Mode) -> Result<Self> {
        let fs = Box::leak(Box::new(FileProvider::new()));
        let mut bundler = Bundler::new(fs, None, ParserOptions::default());
        let stylesheet = bundler.bundle(path)?;
        let css = stylesheet.to_css(PrinterOptions::default())?;
        let source_name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap();

        Ok(match mode {
            Mode::Build => Self {
                source_name,
                build_path: dbg!(digest_filename(&path, &css.code)),
                content: minify::css(&css.code.clone())?,
            },
            Mode::Dev => Self {
                source_name,
                build_path: path.to_owned(),
                content: css.code,
            },
        })
    }
}

pub fn is_buildable_css_file(entry: &DirEntry) -> bool {
    !entry
        .file_name()
        .to_str()
        .is_some_and(|f| f.starts_with("_"))
        && entry.path().extension().is_some_and(|p| p == "css")
}
