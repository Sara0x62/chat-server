use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::structs::{SocketMessage, AppState};

pub fn logging_setup() {
    let filter = EnvFilter::from("website=trace");
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}

pub fn generate_reply(msg_type: &str, msg: &str) -> SocketMessage {
    SocketMessage {
        msg_type: msg_type.to_string(),
        sender: "SERVER".to_string(),
        content: msg.to_string()
    }
}

/*
pub fn check_username(state: &AppState, user: &mut User) -> Option<User> {
    let mut user_set = state.user_set.lock().unwrap();
    let user_out = user.clone();

    if !user_set.contains(user) {
        user_set.insert(user.to_owned());
        Some(user_out)
    } else {
        None
    }
}
*/

pub fn check_user_by_name(state: &AppState, user: &mut String, name: &str) {
    let mut user_set = state.user_set.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());
        user.push_str(name);
    }
}