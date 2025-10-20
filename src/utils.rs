use std::{
    ffi::{OsStr, OsString},
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Result;
use sha1_smol::Sha1;
use walkdir::{DirEntry, WalkDir};

pub fn append_extension(path: &Path, ext: impl AsRef<OsStr>) -> PathBuf {
    let mut os_str: OsString = path.into();
    os_str.push(".");
    os_str.push(ext.as_ref());
    os_str.into()
}

pub fn unprefixed_parent(path: &Path, root: impl AsRef<Path>) -> Option<String> {
    path.strip_prefix(root)
        .ok()?
        .parent()
        .filter(|f| f.components().count() > 0)
        .map(|f| f.to_owned().to_string_lossy().to_string())
}

pub fn split_frontmatter(content: &str) -> Option<(Option<String>, String)> {
    match content
        .split("+++")
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .collect::<Vec<_>>()[..]
    {
        [frontmatter, content] => Some((Some(frontmatter.to_string()), content.to_string())),
        [content] => Some((None, content.to_string())),
        _ => None,
    }
}

pub fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
}

pub fn is_visible(entry: &DirEntry) -> bool {
    !entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.'))
}

pub fn find_files<F>(directory: &Path, filter_files: F) -> impl Iterator<Item = PathBuf> + use<F>
where
    F: Fn(&DirEntry) -> bool,
{
    WalkDir::new(directory)
        .into_iter()
        .filter_entry(is_visible)
        .filter_map(Result::ok)
        .filter(filter_files)
        .map(|f| f.path().to_owned())
}

pub fn new_copy_file(file: PathBuf, root: &Path, out_dir: &Path) -> Result<()> {
    let Some(filename) = file.file_name() else {
        return Err(anyhow::anyhow!("File {file:?} has no filename"));
    };
    let dest = out_dir.join(filename);
    std::fs::create_dir_all(dest.parent().unwrap())?;
    std::fs::copy(root.join(file), dest)?;
    Ok(())
}

pub fn copy_file(root: impl AsRef<Path>, prefix: &str, entry: impl Into<PathBuf>) -> Result<()> {
    let path = entry.into();
    let filename = path.strip_prefix(prefix)?;

    let file = root.as_ref().join(&filename);

    std::fs::create_dir_all(file.parent().unwrap())?;
    File::create(&file)?;
    std::fs::copy(path, file)?;

    Ok(())
}

pub fn write_file(path: &Path, content: impl AsRef<[u8]>) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write::<&Path, &[u8]>(path, content.as_ref())?;
    Ok(())
}

pub fn digest_filename(filename: &Path, content: &str) -> PathBuf {
    let digest = Sha1::from(content).hexdigest();
    let hash = digest.split_at(8).0;
    let Some(extension) = filename.extension() else {
        panic!("No extension found for {filename:?}");
    };

    let path = PathBuf::from(filename).with_extension(hash);
    append_extension(&path, extension)
}

pub fn filename(path: impl Into<PathBuf>) -> String {
    path.into().file_name().map_or_else(
        || panic!("No filename found"),
        |name| name.to_string_lossy().to_string(),
    )
}

pub mod toml_date_jiff_serde {
    use jiff::civil::Date;
    use serde::{self, Deserialize, Deserializer, Serializer};
    use toml::value::Datetime;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Datetime::deserialize(deserializer)?;
        match s.date {
            None => Ok(None),
            Some(date) => Ok(Some(
                Date::new(date.year as i16, date.month as i8, date.day as i8)
                    .map_err(serde::de::Error::custom)?,
            )),
        }
    }

    pub fn serialize<S>(date: &Option<Date>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            None => serializer.serialize_none(),
            Some(date) => serializer.serialize_str(&date.to_string()),
        }
    }
}

pub mod toml_date_deserializer {
    use jiff::civil::Date;
    use serde::{self, Deserializer};

    use super::toml_date_jiff_serde;

    pub fn _deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        match toml_date_jiff_serde::deserialize(deserializer) {
            Ok(None) | Err(_) => Err(serde::de::Error::custom("missing date")),
            Ok(Some(date)) => Ok(date),
        }
    }
}
