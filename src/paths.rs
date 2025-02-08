use std::path::PathBuf;

const OUT_PATH: &str = "_dist";
const TEMPLATE_PATH: &str = "templates";
const PUBLIC_PATH: &str = "public";
const CSS_PATH: &str = "css";
const JS_PATH: &str = "js";
const CONTENT_PATH: &str = "content";

pub const LIVERELOAD_JS: &str = include_str!("livereload.js");

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub out: PathBuf,
    pub templates: PathBuf,
    pub public: PathBuf,
    pub css: PathBuf,
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
            css: root.join(CSS_PATH),
            js: root.join(JS_PATH),
            content: root.join(CONTENT_PATH),
        }
    }
}
