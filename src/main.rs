mod asset;
mod compress;
mod config;
mod constants;
mod content;
mod context;
mod context_builder;
mod minify;
mod render;
mod server;
mod sitemap;
mod utils;
mod watcher;

use std::{env::current_dir, fmt::Display, path::PathBuf, thread, time::Instant};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum, ValueHint};
use serde::Serialize;
use time::UtcOffset;
use tokio::sync::broadcast;
use tracing_subscriber::{
    fmt::time::OffsetTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use crate::{
    config::Config, constants::Paths, context::Metadata, context_builder::ContextBuilder,
    render::Renderer, watcher::start_live_reload,
};

#[derive(Debug, Copy, Clone, ValueEnum, Serialize)]
pub enum Mode {
    Build,
    Dev,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Build => write!(f, "build"),
            Mode::Dev => write!(f, "dev"),
        }
    }
}

impl Mode {
    #[must_use]
    pub const fn is_prod(&self) -> bool {
        matches!(self, Self::Build)
    }

    #[must_use]
    pub const fn is_dev(&self) -> bool {
        matches!(self, Self::Dev)
    }
}

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

    let paths = Paths::new(root);
    let _config = match Config::from_path(&paths.root.join("statisk.toml")) {
        Ok(config) => config,
        Err(_) => bail!("could not find a `statisk.toml` file"),
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

    let mode = match opts.cmd {
        None | Some(Cmds::Dev) => Mode::Dev,
        Some(Cmds::Build) | Some(Cmds::Serve) => Mode::Build,
    };

    let now = Instant::now();

    let metadata = Metadata::new(mode)?;
    let context = ContextBuilder::new(&paths, mode)?.build(&paths, metadata, mode);
    let renderer = Renderer::new(&paths.out);

    renderer.render_context(&context)?;

    let done = now.elapsed();
    tracing::info!(
        "Built {} pages in {:?}ms",
        context.pages.len(),
        done.as_millis()
    );

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
