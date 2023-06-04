// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::error::Error;
use serde::{Deserialize, Serialize};

use crate::{database, generate_ulid};

use super::restricted_string::RestrictedString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: RestrictedString,
}

impl Channel {
    pub fn new(name: RestrictedString) -> Self {
        Self {
            id: generate_ulid(),
            name,
        }
    }

    pub async fn insert(self) -> Result<Self, Error> {
        database().await.create_channel(self).await
    }
}
