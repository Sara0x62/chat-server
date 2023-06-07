use std::{fmt::format, sync::Arc};

use axum::{
    extract::{ws::Message, ws::WebSocket, State, WebSocketUpgrade},
    response::{Html, IntoResponse},
};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::{Mutex, mpsc};
use tracing::info;

use crate::utils::generate_reply;
use crate::{structs::SocketMessage, utils::check_user_by_name, AppState};

pub async fn index() -> Html<&'static str> {
    Html(std::include_str!("../index.html"))
}

pub async fn socket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sink, mut stream) = stream.split();
    let (sends, mut receivers) = mpsc::channel::<String>(16);

    tokio::spawn(async move {
        while let Some(m) = receivers.recv().await {
            if sink.send(m.into()).await.is_err() {
                break;
            }
        }
    });

    let send_task = sends.clone();

    let mut uname: Arc<Mutex<String>> = Arc::new(Mutex::new(String::default()));
    let mut uname2 = Arc::clone(&uname);
    let state2 = Arc::clone(&state);
    let state3 = Arc::clone(&state);
    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            info!("send_task: {:?}", msg);
            let tmp: SocketMessage = serde_json::from_str(&msg).unwrap();

            if tmp.msg_type == "leave" {
                info!("Leave or join message - send task");
                let users = state2.user_set.lock().unwrap();
                let mut user_list = String::new();
                for u in users.iter() {
                    if tmp.msg_type == "leave" && &tmp.content == u {}
                    else {
                        user_list.push_str(&u.to_string());
                        user_list.push('\n');
                    }
                }
                user_list = user_list.strip_suffix('\n').unwrap().to_string();

                let userlist = serde_json::to_string(&SocketMessage {
                    msg_type: "userlist".to_string(),
                    sender: "[Host]".to_string(),
                    content: user_list,
                })
                .unwrap();
                
                let _ = state2.tx.send(format!("{}", userlist));
            }

            if send_task.send(format!("{}", msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let recv_sender = sends.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = stream.next().await {
            
            let mut tmp: SocketMessage = serde_json::from_str(&msg).unwrap();

            tmp.clean_content();
            let reply = serde_json::to_string(&tmp).unwrap();
            // info!("Cleaned content - {}", &tmp.content);

            if tmp.msg_type != "heartbeat" {
                info!("recv_task: {:?}", &msg);
            }

            if tmp.msg_type == "join" {
                if state3.user_set.lock().unwrap().insert(tmp.content.clone()) {
                    tmp.clean_sender();
                    info!("Inserting new user into user set => '{}'", &tmp.content);
                    let mut lock = uname.lock().await;
                    *lock = tmp.content.to_string();
                } else {
                    info!(
                        "Failed to insert new user! - already exists => '{}'",
                        &tmp.content
                    );
                    let reply = generate_reply("invalid_username", "username already in use");
                    let _ = recv_sender.send(serde_json::to_string(&reply).unwrap()).await;
                    return;
                }
            }

            if tmp.msg_type != "heartbeat" && tmp.msg_type != "invalid_username"{
                // Only broadcast if msg is not a heartbeat from client
                info!("not a heartbeat or invalid_username");
                let _ = tx.send(format!("{}", &reply));
            }
            if tmp.msg_type == "join" || tmp.msg_type == "leave" {
                info!("Leave or join message");
                tmp.clean_sender();
                let users = state3.user_set.lock().unwrap();
                let mut user_list = String::new();
                for u in users.iter() {
                    if tmp.msg_type == "leave" && &tmp.content == u {}
                    else {
                        user_list.push_str(&u.to_string());
                        user_list.push('\n');
                    }
                }
                user_list = user_list.strip_suffix('\n').unwrap().to_string();

                let userlist = serde_json::to_string(&SocketMessage {
                    msg_type: "userlist".to_string(),
                    sender: "[Host]".to_string(),
                    content: user_list,
                })
                .unwrap();
                let _ = tx.send(format!("{}", userlist));
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    let mut lock = uname2.lock().await;
    let username: String = lock.to_string();

    if !lock.is_empty() {
        let msg = serde_json::to_string(&SocketMessage {
            msg_type: "leave".to_string(),
            sender: lock.to_string(),
            content: "User left".to_string(),
        })
        .unwrap();
        let _ = state.tx.send(msg);
    }

    /*
    let users = state2.user_set.lock().unwrap();
    let mut user_list = String::new();
    for u in users.iter() {
        if &username == u {}
        else {
            user_list.push_str(&u.to_string());
            user_list.push('\n');
        }
    }
    user_list = user_list.strip_suffix('\n').unwrap().to_string();

    let userlist = serde_json::to_string(&SocketMessage {
        msg_type: "userlist".to_string(),
        sender: "[Host]".to_string(),
        content: user_list,
    })
    .unwrap();
    let _ = state.tx.send(format!("{}", userlist));
    */

    state.user_set.lock().unwrap().remove(&username);
}

