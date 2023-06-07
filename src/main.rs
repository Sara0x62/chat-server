use std::collections::HashSet;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::routing::get;

// Tokio imports
use tokio::{
    sync::broadcast,
};
use tower_http::services::ServeDir;
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
        .nest_service("/styles", ServeDir::new("web/styles"))
        .nest_service("/scripts", ServeDir::new("web/scripts"))
        .with_state(app_state);

    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}