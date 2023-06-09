// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::error::Error;
use serde::{Deserialize, Serialize};

use crate::database;

use super::auth::Service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: u32,
}

#[derive(Debug, Deserialize)]
pub struct UserCreateRequest {
    pub username: String,
    pub service: Service,
    pub oauth_code: String,
}

#[derive(Debug, Serialize)]
pub struct UserLoginResponse {
    pub user: User,
    pub token: String,
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
