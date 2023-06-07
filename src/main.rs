use std::collections::HashSet;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use axum::body::{self, Full};
use axum::http::{Response, HeaderValue};
// Axum imports
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
    Json,
};

use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

// Tokio imports
use tokio::{
    sync::broadcast,
};
use tracing::info;

// Local imports
mod utils;
mod routes;
mod structs;
use crate::structs::*;

#[tokio::main]
async fn main() {
    utils::logging_setup();

    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(16);
    let app_state = Arc::new(
        AppState {
            user_set,
            tx,
        }
    );
    
    let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/websocket", get(routes::socket_handler))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(app_state);

    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}