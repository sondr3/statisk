use std::{path::PathBuf, thread, time::Duration};

use ahash::HashSet;
use anyhow::Result;
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache, new_debouncer,
    notify::{EventKind, RecursiveMode, event::ModifyKind},
};

use crate::{events, events::EventSender, ignorer::StatiskIgnore};

pub fn start_live_reload(
    root: PathBuf,
    events: EventSender,
) -> Result<Debouncer<notify_debouncer_full::notify::RecommendedWatcher, RecommendedCache>> {
    let ignore = StatiskIgnore::new(&root)?;
    let (tx, rx) = flume::unbounded::<DebounceEventResult>();
    let mut watcher = new_debouncer(Duration::from_secs(1), None, tx)?;

    let _ = watcher.watch(root, RecursiveMode::Recursive);

    thread::spawn(move || {
        for event in rx {
            match event {
                Ok(evt) => {
                    evt.into_iter()
                        .filter_map(|e| filter_event(e, &ignore))
                        .flatten()
                        .for_each(|p| {
                            tracing::debug!("File changed: {:?}", p);
                            events.send(events::Event::Path(p));
                        });
                }
                Err(err) => tracing::error!("Error in debouncer: {:?}", err),
            }
        }
    })
    .join()
    .unwrap();

    Ok(watcher)
}

fn filter_event(event: DebouncedEvent, ignore: &StatiskIgnore) -> Option<HashSet<PathBuf>> {
    let paths: HashSet<_> = event
        .paths
        .clone()
        .into_iter()
        .filter(|p| !ignore.is_ignored(p))
        .collect();

    if paths.is_empty() {
        return None;
    }

    match event.kind {
        EventKind::Create(_)
        | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_))
        | EventKind::Remove(_) => Some(paths),
        _ => None,
    }
}
