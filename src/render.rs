use std::path::{Path, PathBuf};

use anyhow::Result;
use url::Url;

use crate::{
    asset::{Asset, PublicFile},
    content::Content,
    context::Context,
    minify::{self},
    sitemap,
    sitemap::UrlEntry,
    utils::{copy_file, write_file},
    BuildMode,
};

pub struct Renderer {
    pub dest: PathBuf,
}

impl Renderer {
    pub fn new(dest: &Path) -> Self {
        Renderer {
            dest: dest.to_path_buf(),
        }
    }

    pub fn render_context(&self, context: &Context) -> Result<()> {
        self.create_dest()?;

        copy_public_files(&context.public_files, &self.dest)?;
        context
            .assets
            .values()
            .try_for_each(|a| write_asset(&self.dest, a))?;

        write_pages(&self.dest, context)?;
        write_sitemap(&self.dest, context)?;

        Ok(())
    }

    fn create_dest(&self) -> Result<()> {
        if self.dest.exists() {
            std::fs::remove_dir_all(&self.dest)?;
        }

        std::fs::create_dir(&self.dest)?;

        Ok(())
    }
}

pub fn write_asset(dest: &Path, asset: &Asset) -> Result<()> {
    write_file(
        &dest.join(asset.build_path.file_name().unwrap()),
        &asset.content,
    )
}

pub fn write_pages(dest: &Path, context: &Context) -> Result<()> {
    write_pages_iter(
        dest,
        context.mode,
        &context.config.url,
        &context,
        context.pages.values(),
    )
}

pub fn write_pages_iter<'a, F>(
    dest: &Path,
    mode: BuildMode,
    url: &Url,
    context: &Context,
    pages: F,
) -> Result<()>
where
    F: Iterator<Item = &'a Content>,
{
    pages.into_iter().try_for_each(|f| {
        write_file(
            &dest.join(&f.out_path),
            if mode.optimize() {
                minify::html(f.render(mode, url, context)?)?
            } else {
                f.render(mode, url, context)?.into()
            },
        )
    })
}

pub fn write_sitemap(dest: &Path, context: &Context) -> Result<()> {
    let urls: Vec<_> = context
        .pages
        .values()
        .filter(|p| !p.frontmatter.special)
        .map(|e| UrlEntry::from_content(e, &context.config.url))
        .collect::<Result<Vec<_>>>()?;

    let sitemap = sitemap::create(urls)?;
    write_file(&dest.join("sitemap.xml"), sitemap)
}

pub fn copy_public_files(files: &[PublicFile], dest: &Path) -> Result<()> {
    files
        .iter()
        .try_for_each(|f| copy_file(dest, &f.prefix, &f.path))
}
