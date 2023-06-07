use std::collections::HashSet;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::Arc;

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

use chat::ChatMessage;
use chat::User;
use futures::SinkExt;
use tokio::sync::broadcast;
use tracing::info;

use futures::stream::StreamExt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use crate::chat::SendMessage;
use crate::chat::UserTmp;

struct AppState {
    user_set: Mutex<HashSet<User>>,
    tx: broadcast::Sender<String>,
}

mod chat;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "website=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState {user_set, tx});

    let app = Router::new()
        .route("/", get(index))
        .route("/websocket", get(websocket_handler))
        .with_state(app_state);

    let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();
    
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str>{
    Html(std::include_str!("../index.html"))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    ws.on_upgrade(
        |socket| websocket(socket, state)
    )
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();

    let mut user: User = User::default();

    while let Some(Ok(Message::Text(msg))) = receiver.next().await {
        info!("Got message {}", msg);
        let t: UserTmp = serde_json::from_str(&msg).unwrap_or_else(|c| UserTmp { name: "default".to_string() });
        
        user.name = t.name;
        break;
    };

    let mut rx = state.tx.subscribe();

    let msg = format!("new user joined - {}", user.name);
    let _ = state.tx.send(msg);

    // Send and recv tasks
    let mut send_task = tokio::spawn(
        async move {
            while let Ok(msg) = rx.recv().await {
                info!("sending msg: {}", msg);
                if sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        }
    );

    let tx = state.tx.clone();
    let user_c = user.clone();

    let mut recv_task = tokio::spawn(
        async move {
            while let Some(Ok(Message::Text(text))) = receiver.next().await {
                let reply: SendMessage = serde_json::from_str(&text).unwrap();

                info!("sending message: {:?}", reply);
                let _ = tx.send(format!("{} : {}", user_c.name, text));
            }
        }
    );

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    let msg = format!("{} left", user.name);
    let _ = state.tx.send(msg);

    state.user_set.lock().unwrap().remove(&user);

}

async fn NewUser(name: String) -> Json<User> {
    let u = User::new_user(name, None);
    Json(u)
}

async fn NewMessage(sender: Uuid, content: String) -> Json<ChatMessage> {
    Json(ChatMessage{sender, content})
}
