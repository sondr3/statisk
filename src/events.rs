use flume::{Receiver, Sender};

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Reload,
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
}
