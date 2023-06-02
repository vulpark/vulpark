use serde::{Deserialize, Serialize};

use crate::structures::message::Message;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseMessage {
    pub _id: String,
    pub author: String,
    pub content: String,
    pub created: String,
}

impl From<&Message> for DatabaseMessage {
    fn from(value: &Message) -> Self {
        Self {
            _id: value.id.to_string(),
            author: value.author.to_string(),
            content: value.content.to_string(),
            created: value.created.to_string(),
        }
    }
}

impl Into<Message> for DatabaseMessage {
    fn into(self) -> Message {
        Message {
            id: self._id,
            author: self.author,
            content: self.content,
            created: self.created,
        }
    }
}
