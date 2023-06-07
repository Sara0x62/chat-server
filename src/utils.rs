use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::structs::SocketMessage;

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