// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::bson::doc;
use mongodb::error::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::structures::user::User;

use super::{
    macros::{basic_fetch, basic_update, eq, id},
    Database,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub _id: String,
    pub username: String,
    pub discriminator: u32,
    pub token: String,
    pub gateway_connected: bool,
}

impl From<DatabaseUser> for User {
    fn from(value: DatabaseUser) -> Self {
        User {
            id: value._id,
            username: value.username,
            discriminator: value.discriminator,
        }
    }
}

impl Database {
    async fn create_user_internal(&self, username: &str) -> Result<Option<DatabaseUser>> {
        let mut discriminator: u32 = rand::thread_rng().gen_range(1..9999);
        let mut count = 1;

        while self
            .users
            .find_one(eq!(username, discriminator), None)
            .await?
            .is_some()
        {
            discriminator = rand::thread_rng().gen_range(1..9999);
            count += 1;
            if count == 9999 {
                return Ok(None);
            }
        }

        let user = DatabaseUser {
            _id: Ulid::new().to_string(),
            username: username.to_string(),
            discriminator,
            token: Ulid::new().to_string(),
            gateway_connected: false,
        };

        self.users.insert_one(user.clone(), None).await?;

        Ok(Some(user))
    }

    pub async fn create_user(&self, username: &str) -> Result<Option<(User, String)>> {
        let Some(user) = self.create_user_internal(username).await? else {
            return Ok(None)
        };

        Ok(Some((user.clone().into(), user.token)))
    }

    pub async fn fetch_user(&self, id: &str) -> Result<Option<User>> {
        basic_fetch!(self.users, id!(id))
    }

    pub async fn fetch_user_login(&self, id: &str) -> Result<Option<(User, String)>> {
        let Some(user): Option<DatabaseUser> = basic_fetch!(self.users, id!(id))? else {
            return Ok(None)
        };

        let token = user.token.clone();

        Ok(Some((user.into(), token)))
    }

    pub async fn fetch_user_token(&self, token: &str) -> Result<Option<User>> {
        basic_fetch!(self.users, eq!(token))
    }

    pub async fn set_user_gateway_connected(
        &self,
        id: &str,
        gateway_connected: bool,
    ) -> Result<Option<User>> {
        basic_update!(self.users, id!(id), eq!(gateway_connected))
    }
}
