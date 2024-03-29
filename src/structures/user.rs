// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::error::Error;
use rweb::Schema;
use serde::{Deserialize, Serialize};

use crate::database;

use super::{auth::Service, restricted_string::RestrictedString};

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: u32,
}

#[derive(Debug, Deserialize, Schema)]
pub struct UserCreateRequest {
    pub username: String,
    pub service: Service,
    pub oauth_code: String,
}

#[derive(Debug, Deserialize, Schema)]
pub struct UserLoginRequest {
    pub service: Service,
    pub oauth_code: String,
}

#[derive(Debug, Serialize, Schema)]
pub struct UserLoginResponse {
    pub user: User,
    pub token: String,
}

impl User {
    pub async fn create(username: &str) -> Result<Option<(Self, String)>, Error> {
        database()
            .await
            .create_user(&RestrictedString::space(username))
            .await
    }
}

impl ToString for User {
    fn to_string(&self) -> String {
        format!("{}:{}", self.username, self.discriminator)
    }
}

impl From<(User, String)> for UserLoginResponse {
    fn from(value: (User, String)) -> Self {
        Self {
            user: value.0,
            token: value.1,
        }
    }
}
