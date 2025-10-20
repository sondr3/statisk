use std::collections::HashMap;

use anyhow::Result;
use time::OffsetDateTime;
use typst::{
    Feature, Features, Library, LibraryExt, World,
    diag::{FileError, FileResult, Warned},
    foundations::{Bytes, Datetime},
    syntax::{FileId, Source},
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
    // fn new(bytes: Vec<u8>, source: Option<Source>) -> Self {
    //     Self {
    //         bytes: Bytes::new(bytes),
    //         source,
    //     }
    // }

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

pub struct TypstContext {
    library: LazyHash<Library>,
    source: Source,
    time: OffsetDateTime,
    files: HashMap<FileId, FileEntry>,
}

impl TypstContext {
    fn new(source: &str) -> TypstContext {
        let library = Library::builder()
            .with_features(Features::from_iter(vec![Feature::Html]))
            .build();

        TypstContext {
            library: LazyHash::new(library),
            source: Source::detached(source),
            time: OffsetDateTime::now_utc(),
            files: HashMap::new(),
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

pub fn render_typst(input: &str) -> Result<String> {
    let world = TypstContext::new(input);
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
