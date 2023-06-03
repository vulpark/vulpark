// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::Deserialize;

use crate::structures::restricted_string::RestrictedString;

use super::Clients;

#[derive(Debug, Deserialize)]
pub struct ChannelCreate {
    name: RestrictedString,
}

pub fn create(token: String, create: ChannelCreate, clients: Clients) {}

pub fn fetch(token: String, id: String) {}
