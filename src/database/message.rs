use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

use crate::structures::message::Message;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseMessage {
    pub _id: String,
    pub author_id: Option<String>,
    pub content: String,
    pub created: DateTime,
}

impl From<&Message> for DatabaseMessage {
    fn from(value: &Message) -> Self {
        Self {
            _id: value.id.to_string(),
            author_id: value.author_id.clone(),
            content: value.content.to_string(),
            created: DateTime::parse_rfc3339_str(value.created.clone()).unwrap(),
        }
    }
}

impl Into<Message> for DatabaseMessage {
    fn into(self) -> Message {
        Message {
            id: self._id,
            author_id: self.author_id,
            content: self.content,
            created: self.created.try_to_rfc3339_string().unwrap(),
        }
    }
}
