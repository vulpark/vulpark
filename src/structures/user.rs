use mongodb::error::Error;
use serde::{Deserialize, Serialize};

use crate::database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: String,
}

impl User {
    pub async fn create(username: String) -> Result<Option<(Self, String)>, Error> {
        database().await.create_user(username).await
    }
}

impl ToString for User {
    fn to_string(&self) -> String {
        format!("{}:{}", self.username, self.discriminator)
    }
}
