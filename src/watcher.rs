use std::{path::Path, thread};

use ahash::HashSet;
use anyhow::{Context, Result};
use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::sync::broadcast::Sender;

use crate::{
    asset::{is_buildable_css_file, Asset},
    context::Context as AppContext,
    context_builder::collect_content,
    paths::Paths,
    render::{write_asset, write_content_iter},
    utils::find_files,
    BuildMode,
};

pub fn start_live_reload(paths: &Paths, context: &AppContext, tx: &Sender<crate::Event>) {
    thread::scope(|scope| {
        let templates = scope.spawn(|| {
            file_watcher(
                &paths.templates.canonicalize()?,
                &["html", "xml"],
                |event| {
                    for path in event.paths.iter().collect::<HashSet<_>>() {
                        content_watch_handler(paths, path, context, tx)?;
                    }
                    Ok(())
                },
            )
        });

        let css = scope.spawn(|| {
            file_watcher(&paths.css.canonicalize()?, &["css"], |event| {
                for path in event.paths.iter().collect::<HashSet<_>>() {
                    css_watch_handler(paths, path, tx)?;
                }
                Ok(())
            })
        });

        let content = scope.spawn(|| {
            file_watcher(&paths.content.canonicalize()?, &["dj", "toml"], |event| {
                for path in event.paths.iter().collect::<HashSet<_>>() {
                    content_watch_handler(paths, path, context, tx)?;
                }
                Ok(())
            })
        });

        css.join().unwrap().unwrap();
        content.join().unwrap().unwrap();
        templates.join().unwrap().unwrap();
    });
}

fn css_watch_handler(paths: &Paths, path: &Path, tx: &Sender<crate::Event>) -> Result<()> {
    tracing::info!(
        "File(s) {:?} changed, rebuilding CSS",
        strip_prefix_paths(&paths.root, path)?
    );
    for file in find_files(&paths.css, is_buildable_css_file) {
        let css = Asset::build_css(&file, BuildMode::Normal)?;
        write_asset(&paths.out, &css)?;
    }

    tx.send(crate::Event::Reload)?;
    Ok(())
}

fn content_watch_handler(
    paths: &Paths,
    path: &Path,
    context: &AppContext,
    tx: &Sender<crate::Event>,
) -> Result<()> {
    tracing::info!(
        "File(s) {:?} changed, rebuilding site",
        strip_prefix_paths(&paths.root, path)?
    );
    let pages = collect_content(paths)?;
    write_content_iter(&paths.out, BuildMode::Normal, context, pages.iter())?;
    tx.send(crate::Event::Reload)?;

    Ok(())
}

fn strip_prefix_paths(prefix: impl AsRef<Path>, path: &Path) -> Result<&Path> {
    path.strip_prefix(prefix.as_ref().canonicalize()?)
        .context("could not strip prefix")
}

fn file_watcher<F, const N: usize>(path: &Path, extensions: &[&str; N], handler: F) -> Result<()>
where
    F: Fn(Event) -> Result<()>,
{
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path, RecursiveMode::Recursive)?;

    for res in rx {
        if let Some(res) = filter_event(res, extensions) {
            handler(res)?;
        }
    }

    Ok(())
}

fn filter_event(res: notify::Result<Event>, extensions: &[&str]) -> Option<Event> {
    match res {
        Ok(event) => match event.kind {
            EventKind::Create(_)
            | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_))
            | EventKind::Remove(_) => event_has_extension(event, extensions),
            _ => None,
        },
        Err(e) => {
            tracing::error!("watch error: {e:?}");
            None
        }
    }
}

fn event_has_extension(event: Event, extensions: &[&str]) -> Option<Event> {
    if event
        .paths
        .iter()
        .any(|p| path_has_extension(p, extensions))
    {
        Some(event)
    } else {
        None
    }
}

fn path_has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .is_some_and(|e| extensions.contains(&e.to_str().unwrap()))
}
