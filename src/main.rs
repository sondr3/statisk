mod asset;
mod build_mode;
mod compress;
mod content;
mod context;
mod context_builder;
mod frontmatter;
mod jotdown;
mod minify;
mod paths;
mod render;
mod server;
mod sitemap;
mod statisk_config;
mod templating;
mod utils;
mod watcher;

use std::{env::current_dir, fs, path::PathBuf, thread, time::Instant};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueHint};
use time::UtcOffset;
use tokio::sync::broadcast;
use tracing_subscriber::{
    fmt::time::OffsetTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use crate::{
    build_mode::BuildMode, context_builder::ContextBuilder, paths::Paths, render::Renderer,
    statisk_config::StatiskConfig, templating::Templates, watcher::start_live_reload,
};

#[derive(Debug, Parser)]
#[command(version, about, author)]
#[clap(args_conflicts_with_subcommands = true)]
struct Options {
    #[arg(long, short)]
    pub verbose: bool,
    /// Directory to run in
    #[arg(value_hint = ValueHint::DirPath, value_name = "dir", global = true)]
    pub dir: Option<PathBuf>,
    /// Configuration management
    #[command(subcommand)]
    pub cmd: Option<Cmds>,
}

#[derive(Subcommand, Debug)]
pub enum Cmds {
    /// start the dev loop
    Dev,
    /// build for production
    Build,
    /// start a local server
    Serve,
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Reload,
    Shutdown,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Options::parse();

    let offset = UtcOffset::current_local_offset().map_or(UtcOffset::UTC, |o| o);
    let format = time::format_description::parse("[hour]:[minute]:[second]")?;
    let timer = OffsetTime::new(offset, format);
    let fmt = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_timer(timer);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!("statisk={}", if opts.verbose { "debug" } else { "info" }).into()
    });

    tracing_subscriber::registry().with(filter).with(fmt).init();

    let root = match opts.dir {
        None => current_dir()?,
        Some(dir) => dir.canonicalize()?,
    };

    let mode = match opts.cmd {
        None | Some(Cmds::Dev) => BuildMode::Normal,
        Some(Cmds::Build) | Some(Cmds::Serve) => BuildMode::Optimized,
    };

    let paths = Paths::new(root);
    if paths.out.exists() {
        tracing::debug!("Removing out directory");
        fs::remove_dir_all(&paths.out)?;
    }

    let config = match StatiskConfig::from_path(&paths.root.join("statisk.toml"), mode) {
        Ok(config) => config,
        Err(err) => bail!("could not read config: {:?}", err),
    };

    match opts.cmd {
        None | Some(Cmds::Dev) => {
            tracing::info!("dev mode engaged...");
        }
        Some(Cmds::Build) => {
            tracing::info!("building for production...");
        }
        Some(Cmds::Serve) => {
            tracing::info!("serving locally...");
        }
    }

    let now = Instant::now();

    let templates = Templates::new(paths.templates.clone())?;
    let context = ContextBuilder::new(&paths, mode)?.build(templates, config, mode);
    let renderer = Renderer::new(&paths.out);

    if matches!(opts.cmd, None | Some(Cmds::Dev | Cmds::Build)) {
        renderer.render_context(&context)?;

        let done = now.elapsed();
        tracing::info!(
            "Built {} pages in {:?}ms",
            context.pages.len(),
            done.as_millis()
        );
    }

    match opts.cmd {
        None | Some(Cmds::Dev) => {
            let (tx, _rx) = broadcast::channel(100);
            let root = paths.out.clone();
            let watcher_tx = tx.clone();
            let watcher = thread::spawn(move || start_live_reload(&paths, &context, &watcher_tx));

            tracing::info!("serving site at http://localhost:3000/...");
            server::create(&root, tx).await?;

            watcher.join().unwrap();
        }
        Some(Cmds::Build) => {
            let now = Instant::now();

            compress::folder(&paths.out)?;

            let done = now.elapsed();
            tracing::info!("Finished compressing output in {:?}ms", done.as_millis());
        }
        Some(Cmds::Serve) => {
            tracing::info!("serving site at http://localhost:3000/...");
            let (tx, _) = broadcast::channel(100);
            server::create(&paths.out, tx).await?;
        }
    }

    Ok(())
}
