use std::{
    fs::File,
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    thread,
    thread::spawn,
};

use anyhow::{Context, Result};
use astra::{Body, ConnectionInfo, Request, Response, ResponseBuilder, Server, Service};
use flume::Receiver;
use http::{StatusCode, header::CONTENT_TYPE, method::Method};
use tungstenite::{Message, WebSocket, accept};

use crate::{
    events::{Event, EventSender},
    utils::extension,
};

pub fn create(root: &Path, events: EventSender) {
    thread::scope(|scope| {
        let server = scope.spawn(|| create_http_server(root).unwrap());
        let websocket = scope.spawn(|| create_websocket_server(&events).unwrap());

        server.join().unwrap();
        websocket.join().unwrap();
    });
}

fn create_http_server(root: &Path) -> Result<()> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));

    let file_serve = FileServe::new(root);
    Server::bind(addr)
        .http1_only()
        .serve(file_serve)
        .context("failed to start server")
}

fn create_websocket_server(events: &EventSender) -> Result<()> {
    let server = NotificationServer::new(events.rx.clone());
    server.start()
}

struct NotificationServer {
    rx: Receiver<Event>,
    clients: Arc<RwLock<Vec<WebSocket<TcpStream>>>>,
}

impl NotificationServer {
    fn new(rx: Receiver<Event>) -> Self {
        NotificationServer {
            rx,
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn start(self) -> Result<()> {
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3001));
        let server = TcpListener::bind(addr)?;

        let connections_events = self.clients.clone();

        spawn(move || {
            while let Ok(event) = self.rx.recv() {
                if let Ok(mut connections) = connections_events.write() {
                    connections.retain_mut(|websocket| {
                        let message = match event {
                            Event::Reload => Message::Text("reload".into()),
                        };

                        if websocket.send(message).is_ok() {
                            true
                        } else {
                            let _ = websocket.close(None);
                            false
                        }
                    });
                }
            }
        });

        for stream in server.incoming().flatten() {
            stream.set_nodelay(true)?;

            let connections = self.clients.clone();

            spawn(move || {
                if let Ok(websocket) = accept(stream)
                    && let Ok(mut connections) = connections.write()
                {
                    connections.push(websocket);
                }
            });
        }

        Ok(())
    }
}

struct FileServe {
    dir: PathBuf,
}

impl Service for FileServe {
    fn call(&self, request: Request, _info: ConnectionInfo) -> Response {
        self.handle(request).unwrap()
    }
}

impl FileServe {
    pub fn new(dir: &Path) -> Self {
        FileServe {
            dir: dir.to_path_buf(),
        }
    }

    fn not_found(&self) -> Result<Response> {
        let file = self.dir.join("404.html");
        if !file.exists() {
            return ResponseBuilder::new()
                .status(StatusCode::NOT_FOUND)
                .body(Body::new("Not found"))
                .context("unable to send");
        }

        let file = File::open(file)?;
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .header(CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::wrap_reader(file))
            .context("unable to send")
    }

    fn empty_not_found(&self) -> Result<Response> {
        ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .context("unable to send")
    }

    fn handle(&self, request: Request) -> Result<Response> {
        if request.method() != Method::GET {
            return ResponseBuilder::new()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::empty())
                .context("unable to send");
        }

        let uri = request.uri();
        let path = PathBuf::from(uri.path());

        if path.extension().is_none() {
            let file = self
                .dir
                .join(uri.path().strip_prefix("/").unwrap_or_default())
                .join("index.html");

            if !file.exists() {
                return self.not_found();
            }

            let file = File::open(file)?;
            return ResponseBuilder::new()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::wrap_reader(file))
                .context("unable to send");
        }

        let path = self
            .dir
            .join(uri.path().strip_prefix("/").unwrap_or_default());

        if !path.exists() {
            return self.empty_not_found();
        }
        let file = File::open(&path)?;

        let mime = new_mime_guess::from_ext(&extension(&path))
            .first_or_text_plain()
            .to_string();
        ResponseBuilder::new()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, mime)
            .body(Body::wrap_reader(file))
            .context("unable to send")
    }
}
