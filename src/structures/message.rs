// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::Utc;
use mongodb::error::Error;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::database;

use super::{channel::Channel, user::User};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub author_id: Option<String>,
    pub content: String,
    pub created: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageCreate {
    pub channel_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub message: Message,
    pub channel: Channel,
    pub author: Option<User>,
}

#[derive(Debug, Deserialize)]
pub struct MessageFetchBefore {
    pub channel: String,
    pub before: String,
    pub max: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MessageFetchAfter {
    pub channel: String,
    pub after: String,
    pub max: Option<i64>,
}

impl Message {
    pub fn new(channel_id: String, author_id: String, content: String) -> Self {
        Message {
            id: Ulid::new().to_string(),
            channel_id,
            author_id: Some(author_id),
            content,
            created: Utc::now().to_rfc3339(),
        }
    }

    pub async fn insert(self) -> Result<Self, Error> {
        database().await.create_message(self).await
    }
}

impl MessageResponse {
    pub async fn from_message(message: Message, channel: Channel) -> Self {
        let Some(id) = &message.author_id else {
            return Self::none(message, channel);
        };

        let Some(user) = database().await.fetch_user(id.clone()).await.unwrap_or(None) else {
            return Self::none(message, channel);
        };

        Self::from(message, channel, Some(user))
    }

    pub fn none(message: Message, channel: Channel) -> Self {
        Self::from(message, channel, None)
    }

    pub fn from(message: Message, channel: Channel, author: Option<User>) -> Self {
        MessageResponse {
            message,
            channel,
            author,
        }
    }
}
