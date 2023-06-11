// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::error::Result;
use serde::{Deserialize, Serialize};

use crate::{database::macros::id, structures::guild::Guild};

use super::{
    macros::{basic_create, basic_fetch},
    Database,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseGuild {
    pub _id: String,
    pub name: String,
}

impl From<&Guild> for DatabaseGuild {
    fn from(value: &Guild) -> Self {
        Self {
            _id: value.id.clone(),
            name: value.name.clone(),
        }
    }
}

impl From<DatabaseGuild> for Guild {
    fn from(value: DatabaseGuild) -> Self {
        Self {
            id: value._id,
            name: value.name,
        }
    }
}

impl Database {
    pub async fn create_guild(&self, guild: Guild) -> Result<Guild> {
        basic_create!(self.guilds, DatabaseGuild::from, guild)
    }

    pub async fn fetch_guild(&self, id: &str) -> Result<Option<Guild>> {
        basic_fetch!(self.guilds, id!(id))
    }
}
