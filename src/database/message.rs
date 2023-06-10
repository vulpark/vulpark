// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::bson::{doc, DateTime};
use mongodb::error::Result;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};

use crate::structures::message::Message;

use super::{
    macros::{after, basic_create, basic_fetch, before, id},
    to_vec, Database,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseMessage {
    pub _id: String,
    pub channel_id: String,
    pub author_id: Option<String>,
    pub content: String,
    pub created: DateTime,
}

impl From<&Message> for DatabaseMessage {
    fn from(value: &Message) -> Self {
        Self {
            _id: value.id.to_string(),
            channel_id: value.channel_id.clone(),
            author_id: value.author_id.clone(),
            content: value.content.to_string(),
            created: DateTime::parse_rfc3339_str(value.created.clone()).unwrap(),
        }
    }
}

impl From<DatabaseMessage> for Message {
    fn from(value: DatabaseMessage) -> Message {
        Message {
            id: value._id,
            channel_id: value.channel_id,
            author_id: value.author_id,
            content: value.content,
            created: value.created.try_to_rfc3339_string().unwrap(),
        }
    }
}

impl Database {
    pub async fn create_message(&self, message: Message) -> Result<Message> {
        basic_create!(self.messages, DatabaseMessage::from, message)?;
        Ok(message)
    }

    pub async fn fetch_message(&self, id: String) -> Result<Option<Message>> {
        Ok(basic_fetch!(self.messages, id!(id)))
    }

    pub async fn fetch_messages_before(
        &self,
        channel_id: String,
        time: String,
        max: i64,
    ) -> Result<Vec<Message>> {
        let Ok(timestamp) = DateTime::parse_rfc3339_str(time) else {
            return Ok(vec![]);
        };

        let Ok(messages) = to_vec(self.messages.find(before!(timestamp, channel_id), FindOptions::builder().limit(max).sort(doc! {"created": -1}).build()).await?).await else {
            return Ok(vec![]);
        };

        Ok(messages.into_iter().map(Into::into).rev().collect())
    }

    pub async fn fetch_messages_after(
        &self,
        channel_id: String,
        time: String,
        max: i64,
    ) -> Result<Vec<Message>> {
        let Ok(timestamp) = DateTime::parse_rfc3339_str(time) else {
            return Ok(vec![]);
        };

        let Ok(messages) = to_vec(self.messages.find(after!(timestamp, channel_id), FindOptions::builder().limit(max).build()).await?).await else {
            return Ok(vec![]);
        };

        Ok(messages.into_iter().map(Into::into).collect())
    }
}
