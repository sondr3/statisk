use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
    asset::PublicFile,
    content::ContentType,
    context::Context,
    minify::{self},
    utils::{copy_file, write_file},
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

        self.copy_public_files(&context.public_files)?;
        self.write_assets(context)?;
        self.write_content(context)?;

        Ok(())
    }

    pub fn write_content(&self, context: &Context) -> Result<()> {
        for page in context.pages.iter() {
            let f = page.value();
            write_file(
                &self.dest.join(&f.out_path),
                match (context.mode.optimize(), f.kind) {
                    (true, ContentType::XML | ContentType::Unknown) => {
                        f.render(context.mode, context)?.into()
                    }
                    (true, ContentType::HTML | ContentType::Typst | ContentType::Jotdown) => {
                        minify::html(&f.render(context.mode, context)?)?
                    }
                    (false, _) => f.render(context.mode, context)?.into(),
                },
            )?;
        }

        Ok(())
    }

    pub fn copy_public_files(&self, files: &[PublicFile]) -> Result<()> {
        files
            .iter()
            .try_for_each(|f| copy_file(&self.dest, &f.prefix, &f.path))
    }

    pub fn write_assets(&self, context: &Context) -> Result<()> {
        for asset in context.assets.iter() {
            let asset = asset.value();
            write_file(
                &self.dest.join(asset.build_path.file_name().unwrap()),
                &asset.content,
            )?;
        }

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
