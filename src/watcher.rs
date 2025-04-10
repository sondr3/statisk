use std::{path::Path, thread};

use ahash::HashSet;
use anyhow::{Context, Result};
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind,
};

use crate::{
    BuildMode,
    asset::{Asset, is_buildable_css_file, is_js},
    context::{Context as AppContext, collect_content, collect_pages},
    paths::Paths,
    utils::find_files,
};

pub fn start_live_reload(paths: &Paths, context: &AppContext) {
    thread::scope(|scope| {
        let templates = scope.spawn(|| {
            file_watcher(
                &paths.templates.canonicalize()?,
                &["html", "xml", "xsl"],
                |event| {
                    for path in event.paths.iter().collect::<HashSet<_>>() {
                        templates_watch_handler(paths, path, context)?;
                    }
                    Ok(())
                },
            )
        });

        let css = scope.spawn(|| {
            file_watcher(&paths.css.canonicalize()?, &["css"], |event| {
                for path in event.paths.iter().collect::<HashSet<_>>() {
                    css_watch_handler(paths, path, context)?;
                }
                Ok(())
            })
        });

        let js = scope.spawn(|| {
            file_watcher(&paths.js.canonicalize()?, &["js", "cjs"], |event| {
                for path in event.paths.iter().collect::<HashSet<_>>() {
                    js_watch_handler(paths, path, context)?;
                }
                Ok(())
            })
        });

        let content = scope.spawn(|| {
            file_watcher(&paths.content.canonicalize()?, &["dj", "toml"], |event| {
                for path in event.paths.iter().collect::<HashSet<_>>() {
                    content_watch_handler(paths, path, context)?;
                }
                Ok(())
            })
        });

        css.join().unwrap().unwrap();
        js.join().unwrap().unwrap();
        content.join().unwrap().unwrap();
        templates.join().unwrap().unwrap();
    });
}

fn css_watch_handler(paths: &Paths, path: &Path, context: &AppContext) -> Result<()> {
    tracing::info!(
        "File(s) {:?} changed, rebuilding CSS",
        strip_prefix_paths(&paths.root, path)?
    );
    for file in find_files(&paths.css, is_buildable_css_file) {
        let css = Asset::build_css(&file, BuildMode::Normal)?;
        context.update_asset(css.source_name.clone(), css)?;
    }

    Ok(())
}

fn js_watch_handler(paths: &Paths, path: &Path, context: &AppContext) -> Result<()> {
    tracing::info!(
        "File(s) {:?} changed, rebuilding JS",
        strip_prefix_paths(&paths.root, path)?
    );
    for file in find_files(&paths.js, is_js) {
        let js = Asset::build_js(&file, BuildMode::Normal)?;
        context.update_asset(js.source_name.clone(), js)?;
    }

    Ok(())
}

fn content_watch_handler(paths: &Paths, path: &Path, context: &AppContext) -> Result<()> {
    tracing::info!(
        "Content {:?} changed, rebuilding...",
        strip_prefix_paths(&paths.root, path)?
    );
    for page in collect_content(paths)? {
        context.update_page(page.filename(), page)?;
    }

    Ok(())
}

fn templates_watch_handler(paths: &Paths, path: &Path, context: &AppContext) -> Result<()> {
    tracing::info!(
        "Template {:?} changed, rebuilding...",
        strip_prefix_paths(&paths.root, path)?
    );
    for page in collect_pages(paths)? {
        context.update_page(page.filename(), page)?;
    }

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
