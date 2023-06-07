// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use crate::with_lock;

use super::{
    channel::Channel,
    client::{Client, ClientHolder},
    message::Message,
    user::User,
};

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

impl ReceivedEvent {
    pub async fn handle(&self, mut client: Client, clients: &ClientHolder) -> Option<Event> {
        match self {
            Self::Handshake { token } => {
                if let Some(_) = client.user_id {
                    return None;
                }
                let user = client.set_user(token.clone()).await?;
                {
                    let mut lock = with_lock!(clients);
                    if let Some(clients) = lock.get_mut(&user.id) {
                        clients.push(client.clone());
                    } else {
                        lock.insert(user.id.clone(), vec![client.clone()]);
                    };
                }
                Some(Event::HandshakeComplete { user })
            }
        }
    }
}
