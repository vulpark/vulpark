// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::error::Result;
use serde::{Deserialize, Serialize};

use crate::structures::channel::{Channel, ChannelLocation};

use super::{
    macros::{basic_create, basic_fetch, id},
    Database,
};

#[derive(Serialize, Deserialize)]
pub struct DatabaseChannel {
    pub _id: String,
    pub name: String,
    pub location: ChannelLocation,
}

impl From<&Channel> for DatabaseChannel {
    fn from(value: &Channel) -> Self {
        Self {
            _id: value.id.to_string(),
            name: value.name.clone(),
            location: value.location.clone(),
        }
    }
}

impl From<DatabaseChannel> for Channel {
    fn from(value: DatabaseChannel) -> Self {
        Channel {
            id: value._id,
            name: value.name,
            location: value.location,
        }
    }
}

impl Database {
    pub async fn create_channel(&self, channel: Channel) -> Result<Channel> {
        basic_create!(self.channels, DatabaseChannel::from, channel)
    }

    pub async fn fetch_channel(&self, id: String) -> Result<Option<Channel>> {
        basic_fetch!(self.channels, id!(id))
    }
}
