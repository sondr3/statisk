mod asset;
mod build_mode;
mod cli;
mod compress;
mod config;
mod content;
mod context;
mod events;
mod frontmatter;
mod jotdown;
mod lua;
mod meta;
mod minify;
mod render;
mod server;
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
    lua::{create_lua_context, statisk::LuaStatisk},
    render::Renderer,
    templating::Templates,
    utils::walk_ignored,
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

    let lua = create_lua_context(mode, root.clone())?;
    let statisk = match LuaStatisk::load(&lua, &root.join("statisk.lua")) {
        Ok(statisk) => statisk,
        Err(err) => bail!("could not read config: {:?}", err),
    };
    let paths = statisk.paths.clone();
    dbg!(&statisk);

    let files: Vec<_> = walk_ignored(&root)
        .inspect(|p| {
            for output in &statisk.outputs {
                let is_match = output.is_match(p);
                let pattern = output.glob_pattern();
                println!("matcher {pattern:?} on file {p:?}: {is_match}");
            }
        })
        .collect();
    //
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
    let templates = Templates::new(&statisk.template_root())?;
    let renderer = Renderer::new(&statisk.out_dir());
    let mut context = Context::new(templates, statisk, renderer, mode, events.clone());
    context.collect(&paths)?;

    if matches!(opts.cmd, None | Some(Cmds::Dev | Cmds::Build)) {
        if paths.out_dir.exists() {
            tracing::debug!("Removing out directory");
            fs::remove_dir_all(&paths.out_dir)?;
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
            let root = paths.out_dir.clone();
            let watcher = thread::spawn(move || start_live_reload(&paths, &context));

            tracing::info!("serving site at http://localhost:3000/...");
            server::create(&root, events.clone());

            watcher.join().unwrap();
        }
        Some(Cmds::Build) => {
            let now = Instant::now();

            compress::folder(&paths.out_dir)?;

            let done = now.elapsed();
            tracing::info!("Finished compressing output in {:?}ms", done.as_millis());
        }
        Some(Cmds::Serve) => {
            tracing::info!("serving site at http://localhost:3000/...");
            server::create(&paths.out_dir, events.clone());
        }
        Some(Cmds::Completion { .. }) => unreachable!(),
    }

    Ok(())
}
