use std::sync::Arc;

use axum::{
    extract::{ws::Message, ws::WebSocket, State, WebSocketUpgrade},
    response::{Html, IntoResponse},
};
use futures::{SinkExt, StreamExt};
use tokio::sync::{Mutex, mpsc};
use tracing::info;

use crate::utils::generate_reply;
use crate::{structs::SocketMessage, AppState};

pub async fn index() -> Html<&'static str> {
    Html(std::include_str!("../web/index.html"))
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

    let username: Arc<Mutex<String>> = Arc::new(Mutex::new(String::default()));
    let username_arc = Arc::clone(&username);
    let sender_state = Arc::clone(&state);
    let receiver_state = Arc::clone(&state);
    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {

            let tmp: SocketMessage = serde_json::from_str(&msg).unwrap();
            info!("send_task: {:?}", &tmp);

            // A user left, update userlist
            if tmp.msg_type == "leave" {
                info!("User left");
                let users = sender_state.user_set.lock().unwrap();
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
                
                let _ = sender_state.tx.send(format!("{}", userlist));
            }

            if send_task.send(format!("{}", msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let recv_sender = sends.clone();

    // Receive from client, broadcast to the other clients
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = stream.next().await {
            
            // Convert string to our SocketMessage struct
            let mut tmp: SocketMessage = serde_json::from_str(&msg).unwrap();

            // Sanitize inputs
            tmp.clean_content();
            let reply = serde_json::to_string(&tmp).unwrap();

            // Filter out heartbeat spam from the logs
            if tmp.msg_type != "heartbeat" {
                info!("recv_task: {:?}", &tmp);
            }

            // New user?
            if tmp.msg_type == "join" {
                if receiver_state.user_set.lock().unwrap().insert(tmp.content.clone()) {
                    // User added to active userlist
                    info!("Inserting new user into user set => '{}'", &tmp.content);
                    let mut lock = username.lock().await;
                    *lock = tmp.content.to_string();

                    // Broadcast user joined message
                    let _ = tx.send(reply);

                    // Collect updated userlist
                    let users = receiver_state.user_set.lock().unwrap();
                    let mut user_list = String::new();

                    for u in users.iter() {
                        user_list.push_str(&u.to_string());
                        user_list.push('\n');
                    } // Get rid of trailing newline
                    user_list = user_list.strip_suffix('\n').unwrap().to_string();

                    // Broadcast new userlist
                    let userlist = serde_json::to_string(&SocketMessage {
                        msg_type: "userlist".to_string(),
                        sender: "[Host]".to_string(),
                        content: user_list,
                    }).unwrap();

                    let _ = tx.send(format!("{}", userlist));
                } else {
                    // Unable to add user; break connection with client
                    info!(
                        "Failed to insert new user! - already exists => '{}'",
                        &tmp.content
                    );
                    let new_reply = generate_reply("invalid_username", "username already in use");
                    let _ = recv_sender.send(serde_json::to_string(&new_reply).unwrap()).await;
                    return;
                }
            } 
            else if tmp.msg_type == "message" {
                let clientname = username.lock().await;

                // Check if the JSON sender is correct
                if tmp.sender != *clientname {
                    let new_reply = generate_reply("bad_request", "Username credentials dont match");
                    let _ = recv_sender.send(serde_json::to_string(&new_reply).unwrap()).await;
                } else {
                    let _ = tx.send(format!("{}", &reply));
                }
            }
            // If message is anything but a heartbeat and none of the above
            // Broadcast to other users
            else if tmp.msg_type != "heartbeat" {
                // Only broadcast if msg is not a heartbeat from client
                info!("not a heartbeat");
                let _ = tx.send(format!("{}", &reply));
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    let lock = username_arc.lock().await;
    let username: String = lock.to_string();

    // Remove user, if it was added, from the userlist
    state.user_set.lock().unwrap().remove(&username);

    // If the username was actually set,
    // This runs when a user leaves
    if !lock.is_empty() {
        let msg = serde_json::to_string(&SocketMessage {
            msg_type: "leave".to_string(),
            sender: lock.to_string(),
            content: "User left".to_string(),
        })
        .unwrap();
        let _ = state.tx.send(msg);

        // Unsure why this will not work
        // Causes PoisonError
        // @ src/routes.rs:103:51
        // @ src/routes.rs:172:27
        // -----
        /*
        info!("User left");
        let users = state.user_set.lock().unwrap();
        let mut user_list = String::new();
        for u in users.iter() {
            user_list.push_str(&u.to_string());
            user_list.push('\n');
        }
        user_list = user_list.strip_suffix('\n').unwrap().to_string();

        let userlist = serde_json::to_string(&SocketMessage {
            msg_type: "userlist".to_string(),
            sender: "[Host]".to_string(),
            content: user_list,
        }).unwrap();
        
        // Broadcast new userlist to remaining clients
        let _ = state.tx.send(format!("{}", userlist));
        */
    }
}

