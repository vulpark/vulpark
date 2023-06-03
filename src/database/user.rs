// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::bson::doc;
use mongodb::error::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::structures::user::User;

use super::{macros::*, Database};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub(super) _id: String,
    pub(super) username: String,
    pub(super) discriminator: u32,
    pub(super) token: String,
}

impl Into<User> for DatabaseUser {
    fn into(self) -> User {
        User {
            id: self._id,
            username: self.username,
            discriminator: self.discriminator.to_string(),
        }
    }
}

impl Database {
    async fn create_user_internal(&self, username: String) -> Result<Option<DatabaseUser>> {
        let mut discrim: u32 = rand::thread_rng().gen_range(1..9999);
        let mut count = 1;
        while let Some(_) = self
            .users
            .find_one(
                doc! {"username": username.clone(), "discriminator": discrim},
                None,
            )
            .await?
        {
            discrim = rand::thread_rng().gen_range(1..9999);
            count += 1;
            if count == 9999 {
                return Ok(None);
            }
        }
        let user = DatabaseUser {
            _id: Ulid::new().to_string(),
            username,
            discriminator: discrim,
            token: Ulid::new().to_string(),
        };
        self.users.insert_one(user.clone(), None).await?;
        Ok(Some(user))
    }

    pub async fn create_user(&self, username: String) -> Result<Option<(User, String)>> {
        let Some(user) = self.create_user_internal(username).await? else {
            return Ok(None)
        };
        Ok(Some((user.clone().into(), user.token)))
    }

    pub async fn fetch_user(&self, id: String) -> Result<Option<User>> {
        Ok(basic_fetch!(self.users, id!(id)))
    }

    pub async fn fetch_user_token(&self, token: String) -> Result<Option<User>> {
        Ok(basic_fetch!(self.users, eq!("token", token)))
    }
}
