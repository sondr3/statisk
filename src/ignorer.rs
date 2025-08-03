use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::{
    WalkBuilder,
    gitignore::{Gitignore, GitignoreBuilder, gitconfig_excludes_path},
};
use walkdir::WalkDir;

use crate::utils::is_visible;

pub struct StatiskIgnore {
    ignore: Gitignore,
}

impl StatiskIgnore {
    pub fn new(root: &Path) -> Result<Self> {
        let mut ignore_builder = GitignoreBuilder::new(root);
        if root.join(".statiskignore").exists() {
            ignore_builder.add(root.join(".statiskignore"));
        }
        if root.join(".gitignore").exists() {
            ignore_builder.add(root.join(".gitignore"));
        }
        if gitconfig_excludes_path().is_some_and(|p| p.exists()) {
            ignore_builder.add(gitconfig_excludes_path().unwrap());
        }

        let ignore = ignore_builder.build()?;
        Ok(StatiskIgnore { ignore })
    }

    pub fn walker(root: &Path) -> impl Iterator<Item = PathBuf> {
        WalkBuilder::new(root)
            .add_custom_ignore_filename(".statiskignore")
            .build()
            .filter_map(Result::ok)
            .map(|f| f.path().to_owned())
            .map(move |f| f.strip_prefix(root).unwrap().to_owned())
    }

    pub fn walk_dir(directory: &Path) -> impl Iterator<Item = PathBuf> {
        WalkDir::new(directory)
            .into_iter()
            .filter_entry(is_visible)
            .filter_map(Result::ok)
            .map(|f| f.path().to_owned())
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        let is_dir = path.is_dir();
        self.ignore
            .matched_path_or_any_parents(path, is_dir)
            .is_ignore()
    }
}
