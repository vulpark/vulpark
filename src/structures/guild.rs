// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use crate::{database, generate_ulid};

use super::{restricted_string::RestrictedString, user::User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub owner_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GuildCreate {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildResponse {
    guild: Guild,
    owner: User,
}

impl Guild {
    pub fn new(name: &str, owner_id: &str) -> Self {
        Self {
            id: generate_ulid(),
            name: RestrictedString::space(name),
            owner_id: owner_id.to_string(),
        }
    }

    pub async fn insert(self) -> mongodb::error::Result<Self> {
        let guild = database().await.create_guild(self).await?;
        database()
            .await
            .join_guild(&guild.id, &guild.owner_id)
            .await?;
        Ok(guild)
    }
}

impl GuildResponse {
    pub async fn from(guild: Guild) -> mongodb::error::Result<Self> {
        let owner = database().await.fetch_user(&guild.owner_id).await?.unwrap();
        Ok(Self { guild, owner })
    }

    pub fn new(guild: Guild, owner: User) -> Self {
        Self { guild, owner }
    }
}
