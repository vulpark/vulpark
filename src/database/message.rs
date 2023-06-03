use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};
use mongodb::options::FindOptions;
use mongodb::error::Result;

use crate::structures::message::Message;

use super::{Database, to_vec, macros::*};

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

impl Database {
    pub async fn create_message(&self, message: Message) -> Result<Message> {
        self.messages
            .insert_one(DatabaseMessage::from(&message), None)
            .await?;
        Ok(message)
    }

    pub async fn fetch_message(&self, id: String) -> Result<Option<Message>> {
        Ok(basic_fetch!(self.messages, id!(id)))
    }

    pub async fn fetch_messages_before(
        &self,
        time: String,
        max: i64,
    ) -> Result<Vec<Message>> {
        let Ok(timestamp) = DateTime::parse_rfc3339_str(time) else {
            return Ok(vec![]);
        };

        let Ok(messages) = to_vec(self.messages.find(before!(timestamp), FindOptions::builder().limit(max).sort(doc! {"created": -1}).build()).await?).await else {
            return Ok(vec![]);
        };

        Ok(messages.into_iter().map(|it| it.into()).rev().collect())
    }

    pub async fn fetch_messages_after(
        &self,
        time: String,
        max: i64,
    ) -> Result<Vec<Message>> {
        let Ok(timestamp) = DateTime::parse_rfc3339_str(time) else {
            return Ok(vec![]);
        };

        let Ok(messages) = to_vec(self.messages.find(after!(timestamp), FindOptions::builder().limit(max).build()).await?).await else {
            return Ok(vec![]);
        };

        Ok(messages.into_iter().map(|it| it.into()).collect())
    }
}
