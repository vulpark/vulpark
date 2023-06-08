// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use super::{channel::Channel, message::Message, user::User};

#[derive(Serialize)]
pub enum Event {
    HandshakeStart {},
    HandshakeComplete {
        user: User,
    },
    MessageCreate {
        message: Message,
        author: Option<User>,
        channel: Channel,
    },
    ChannelCreate {
        channel: Channel,
        creator: User,
    },
}

#[derive(Debug, Deserialize)]
pub enum ReceivedEvent {
    Handshake { token: String },
}

impl ToString for Event {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
