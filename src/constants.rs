use std::path::PathBuf;

pub const OUT_PATH: &str = "_dist";
pub const TEMPLATE_PATH: &str = "templates";
pub const PUBLIC_PATH: &str = "public";
pub const CSS_PATH: &str = "styles";
pub const JS_PATH: &str = "js";
pub const CONTENT_PATH: &str = "content";

pub const LIVERELOAD_JS: &str = include_str!("livereload.js");

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub out: PathBuf,
    pub templates: PathBuf,
    pub public: PathBuf,
    pub styles: PathBuf,
    pub js: PathBuf,
    pub content: PathBuf,
}

impl Paths {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: root.clone(),
            out: root.join(OUT_PATH),
            templates: root.join(TEMPLATE_PATH),
            public: root.join(PUBLIC_PATH),
            styles: root.join(CSS_PATH),
            js: root.join(JS_PATH),
            content: root.join(CONTENT_PATH),
        }
    }
}
