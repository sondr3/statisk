mod asset;
mod build_mode;
mod cli;
mod compress;
mod content;
mod context;
mod events;
mod frontmatter;
mod jotdown;
mod lua;
mod minify;
mod paths;
mod render;
mod server;
mod statisk_config;
mod templating;
mod typst;
mod utils;
mod watcher;

use std::{env::current_dir, fs, thread, time::Instant};

use anyhow::{Result, bail};
use clap::{CommandFactory, Parser};
use time::UtcOffset;
use tracing_subscriber::{
    EnvFilter, fmt::time::OffsetTime, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::{
    build_mode::BuildMode,
    cli::{Cmds, Options, print_completion},
    context::Context,
    events::EventSender,
    lua::lua_context,
    paths::Paths,
    render::Renderer,
    statisk_config::StatiskConfig,
    templating::Templates,
    watcher::start_live_reload,
};

fn main() -> Result<()> {
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
        None | Some(Cmds::Dev | Cmds::Completion { .. }) => BuildMode::Normal,
        Some(Cmds::Build | Cmds::Serve) => BuildMode::Optimized,
    };

    let paths = Paths::new(&root);
    let config = match StatiskConfig::from_path(&paths.root.join("statisk.toml"), mode) {
        Ok(config) => config,
        Err(err) => bail!("could not read config: {:?}", err),
    };

    lua_context(&paths.root.join("statisk.lua"))?;

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
        Some(Cmds::Completion { shell }) => {
            let mut app = Options::command();
            print_completion(shell, &mut app);
            return Ok(());
        }
    }

    let now = Instant::now();

    let events = EventSender::new();
    let templates = Templates::new(&paths.templates)?;
    let renderer = Renderer::new(&paths.out);
    let mut context = Context::new(templates, config, renderer, mode, events.clone());
    context.collect(&paths)?;

    if matches!(opts.cmd, None | Some(Cmds::Dev | Cmds::Build)) {
        if paths.out.exists() {
            tracing::debug!("Removing out directory");
            fs::remove_dir_all(&paths.out)?;
        }

        context.build()?;

        let done = now.elapsed();
        tracing::info!(
            "Built {} pages in {:?}ms",
            context.pages.len(),
            done.as_millis()
        );
    }

    match opts.cmd {
        None | Some(Cmds::Dev) => {
            let root = paths.out.clone();
            let watcher = thread::spawn(move || start_live_reload(&paths, &context));

            tracing::info!("serving site at http://localhost:3000/...");
            server::create(&root, events.clone());

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
            server::create(&paths.out, events.clone());
        }
        Some(Cmds::Completion { .. }) => unreachable!(),
    }

    Ok(())
}
