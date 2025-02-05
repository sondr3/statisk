use std::path::PathBuf;

pub const OUT_PATH: &str = "./_dist";
pub const ROOT_PATH: &str = "./";
pub const TEMPLATE_PATH: &str = "./templates";
pub const PUBLIC_PATH: &str = "./public";
pub const CSS_PATH: &str = "./styles";
pub const JS_PATH: &str = "./js";
pub const CONTENT_PATH: &str = "./content";

#[derive(Debug, Clone)]
pub struct Paths {
    pub out: PathBuf,
    pub root: PathBuf,
    pub templates: PathBuf,
    pub public: PathBuf,
    pub styles: PathBuf,
    pub js: PathBuf,
    pub content: PathBuf,
}

impl Paths {
    pub fn new() -> Self {
        Self {
            out: PathBuf::from(OUT_PATH),
            root: PathBuf::from(ROOT_PATH),
            templates: PathBuf::from(TEMPLATE_PATH),
            public: PathBuf::from(PUBLIC_PATH),
            styles: PathBuf::from(CSS_PATH),
            js: PathBuf::from(JS_PATH),
            content: PathBuf::from(CONTENT_PATH),
        }
    }
}
