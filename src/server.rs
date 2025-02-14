#![allow(clippy::unused_async)]

use std::{
    net::{Ipv4Addr, SocketAddr},
    path::Path,
};

use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::{net::TcpListener, sync::broadcast::Sender};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::Event;

pub async fn create(root: &Path, tx: Sender<Event>) -> Result<()> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    let listener = TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        router(root, tx).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("Failed to start server")
}

fn router(root: &Path, tx: Sender<Event>) -> Router {
    Router::new()
        .fallback_service(
            ServeDir::new(root)
                .append_index_html_on_directories(true)
                .not_found_service(ServeFile::new(root.join("404/index.html"))),
        )
        .route("/_ws", get(ws_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(tx)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(tx): State<Sender<Event>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, tx, addr))
}

async fn handle_socket(mut socket: WebSocket, tx: Sender<Event>, addr: SocketAddr) {
    tracing::debug!("{addr} connected");
    let mut rx = tx.subscribe();

    while let Ok(event) = rx.recv().await {
        let res = match event {
            Event::Reload => socket.send(Message::Text("reload".into())).await,
            Event::Shutdown => socket.send(Message::Text("shutdown".into())).await,
        };

        if res.is_err() {
            // disconnect client but don't warn about it
            break;
        }
    }
}
