use std::path::PathBuf;

use flume::{Receiver, Sender};

#[derive(Debug, Clone)]
pub enum Event {
    Reload,
    Path(PathBuf),
}

pub fn events() -> (Sender<Event>, Receiver<Event>) {
    flume::unbounded::<Event>()
}

#[derive(Debug, Clone)]
pub struct EventSender {
    pub tx: Sender<Event>,
    pub rx: Receiver<Event>,
}

impl EventSender {
    pub fn new() -> Self {
        let (tx, rx) = events();
        EventSender { tx, rx }
    }

    pub fn send(&self, event: Event) {
        self.tx.send(event).expect("Failed to send event");
    }
}
