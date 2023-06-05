// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::Utc;
use mongodb::error::Error;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub author_id: Option<String>,
    pub content: String,
    pub created: String,
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
