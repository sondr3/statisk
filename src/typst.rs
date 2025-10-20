use std::{
    collections::{HashMap, hash_map::Entry},
    env::current_dir,
    fs,
    path::Path,
    sync::Mutex,
};

use anyhow::Result;
use time::OffsetDateTime;
use typst::{
    Feature, Features, Library, LibraryExt, World,
    diag::{FileError, FileResult, Warned},
    foundations::{Bytes, Datetime},
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_html::HtmlDocument;

#[derive(Clone, Debug)]
struct FileEntry {
    bytes: Bytes,
    source: Option<Source>,
}

impl FileEntry {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Bytes::new(bytes),
            source: None,
        }
    }

    fn source(&mut self, id: FileId) -> FileResult<Source> {
        let source = if let Some(source) = &self.source {
            source
        } else {
            let contents = std::str::from_utf8(&self.bytes).map_err(|_| FileError::InvalidUtf8)?;
            let contents = contents.trim_start_matches('\u{feff}');
            let source = Source::new(id, contents.into());
            self.source.insert(source)
        };
        Ok(source.clone())
    }
}

#[derive(Debug)]
struct FileSystem {
    files: Mutex<HashMap<FileId, FileEntry>>,
}

impl FileSystem {
    fn new() -> Self {
        Self {
            files: Mutex::new(HashMap::new()),
        }
    }

    fn map_file<T: Clone>(
        &self,
        id: FileId,
        f: impl FnOnce(&mut FileEntry) -> FileResult<T>,
    ) -> FileResult<T> {
        let mut files = self.files.lock().unwrap();

        match files.entry(id) {
            Entry::Occupied(entry) => Ok(f(entry.into_mut())?),
            Entry::Vacant(entry) => {
                let path = id
                    .vpath()
                    .resolve(&current_dir().unwrap())
                    .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))?;

                let bytes = fs::read(&path).map_err(|e| FileError::from_io(e, &path))?;

                let mut file_entry = FileEntry::new(bytes);

                let result = f(&mut file_entry)?;

                entry.insert(file_entry);

                Ok(result)
            }
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.map_file(id, |file_entry| Ok(Bytes::clone(&file_entry.bytes)))
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.map_file(id, |file_entry| file_entry.source(id))
    }
}

pub struct TypstContext {
    library: LazyHash<Library>,
    source: Source,
    time: OffsetDateTime,
    files: HashMap<FileId, FileEntry>,
}

impl TypstContext {
    fn new(source: &str, files: HashMap<FileId, FileEntry>) -> TypstContext {
        let library = Library::builder()
            .with_features(Features::from_iter(vec![Feature::Html]))
            .build();

        TypstContext {
            library: LazyHash::new(library),
            source: Source::detached(source),
            time: OffsetDateTime::now_utc(),
            files,
        }
    }

    fn sandbox_file(&self, id: FileId) -> FileResult<&FileEntry> {
        if let Some(entry) = self.files.get(&id) {
            Ok(entry)
        } else {
            Err(FileError::NotFound(
                id.vpath().as_rootless_path().to_path_buf(),
            ))
        }
    }
}

impl World for TypstContext {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        unimplemented!("Not used for HTML rendering")
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            self.sandbox_file(id)?.clone().source(id)
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.sandbox_file(id).map(|file| file.bytes.clone())
    }

    fn font(&self, _index: usize) -> Option<Font> {
        unimplemented!("Not used for HTML rendering")
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

pub fn render_typst(input: &str, source: &Path) -> Result<String> {
    // Load all _*.typ files from the source directory
    let mut files = HashMap::new();

    if let Some(dir) = source.parent()
        && let Ok(entries) = fs::read_dir(dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();

            // Check if file starts with _ and ends with .typ
            if let Some(filename) = path.file_name().and_then(|n| n.to_str())
                && filename.starts_with('_')
                && filename.ends_with(".typ")
            {
                // Read file contents
                if let Ok(bytes) = fs::read(&path) {
                    // Create a virtual path relative to the source directory
                    let vpath = VirtualPath::new(filename);
                    let file_id = FileId::new(None, vpath);

                    files.insert(file_id, FileEntry::new(bytes));
                }
            }
        }
    }

    let world = TypstContext::new(input, files);
    let Warned { output, warnings } = typst::compile::<HtmlDocument>(&world);

    for warning in warnings {
        if !warning.message.contains("html export") {
            eprintln!("Warning: {:?}", warning);
        }
    }

    let output = match output {
        Ok(output) => output,
        Err(err) => anyhow::bail!("Failed to compile Typst document: {:?}", err),
    };

    let result = match typst_html::html(&output) {
        Ok(res) => res,
        Err(err) => anyhow::bail!("Failed to render Typst document: {:?}", err),
    };

    let start = result.find("<body>").unwrap();
    let end = result.rfind("</body>").unwrap();

    let result = result[start + 6..end].to_string();
    Ok(result)
}
