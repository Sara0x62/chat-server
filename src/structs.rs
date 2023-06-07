use std::sync::Mutex;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct User {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketMessage {
    pub msg_type: String,
    pub sender: String,
    pub content: String,
}

impl SocketMessage {
    pub fn clean_content(&mut self) {
        self.content = self.content.
                replace("<", "&lt;")
                .replace(">", "&gt;")
                .replace("\"", "&quot;")
                .replace("'", "&#039;");
        self.sender = self.sender.
            replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#039;");
    }
}

pub struct AppState {
    pub user_set: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<String>
}
