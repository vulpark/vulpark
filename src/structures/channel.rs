// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use super::restricted_string::RestrictedString;

#[derive(Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: RestrictedString,
}
