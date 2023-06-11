// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{database, generate_ulid};

use super::restricted_string::RestrictedString;

pub struct Guild {
    pub id: String,
    pub name: String,
}

impl Guild {
    pub fn new(name: &str) -> Self {
        Self {
            id: generate_ulid(),
            name: RestrictedString::space(name),
        }
    }

    pub async fn insert(self) -> mongodb::error::Result<Self> {
        database().await.create_guild(self).await
    }
}
