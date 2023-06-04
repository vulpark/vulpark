// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::error::Result;
use serde::{Deserialize, Serialize};

use crate::structures::{channel::Channel, restricted_string::RestrictedString};

use super::{macros::*, Database};

#[derive(Serialize, Deserialize)]
pub struct DatabaseChannel {
    pub _id: String,
    pub name: RestrictedString,
}

impl From<&Channel> for DatabaseChannel {
    fn from(value: &Channel) -> Self {
        Self {
            _id: value.id.to_string(),
            name: value.name.clone(),
        }
    }
}

impl Into<Channel> for DatabaseChannel {
    fn into(self) -> Channel {
        Channel {
            id: self._id,
            name: self.name,
        }
    }
}

impl Database {
    pub async fn create_channel(&self, channel: Channel) -> Result<Channel> {
        basic_create!(self.channels, DatabaseChannel::from, channel)?;
        Ok(channel)
    }

    pub async fn fetch_channel(&self, id: String) -> Result<Option<Channel>> {
        Ok(basic_fetch!(self.channels, id!(id)))
    }
}
