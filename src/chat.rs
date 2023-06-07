use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct User {
    pub uuid: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct UserTmp {
    pub name: String,
}


#[derive(Deserialize, Debug, Serialize)]
pub struct ChatMessage {
    pub sender: Uuid,
    pub content: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SendMessage {
    pub sender: String,
    pub content: String,
}


const DEFAULT_COLOR: &'static str = "0xff0000";
impl User {
    pub fn new_user(name: String, color: Option<String>) -> User {
        let uuid = Uuid::new_v4();
        let color = color.unwrap_or(String::from(DEFAULT_COLOR));
        User {
            uuid,
            name,
            color,
        }
    }

    pub fn default() -> User {
        let uuid = Uuid::new_v4();
        let color = String::from(DEFAULT_COLOR);
        let name = String::from("default");
        User {
            uuid,
            name,
            color,
        }
    }
}