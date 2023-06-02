use chrono::Utc;
use mongodb::error::Error;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub author_id: Option<String>,
    pub content: String,
    pub created: String,
}

impl Message {
    pub fn from_user(author_id: String, content: String) -> Self {
        Message {
            id: Ulid::new().to_string(),
            author_id: Some(author_id),
            content,
            created: Utc::now().to_rfc3339(),
        }
    }

    pub async fn insert(self) -> Result<Self, Error> {
        database().await.create_message(self).await
    }
}
