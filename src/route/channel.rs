// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

use crate::{
    database,
    structures::{channel::{Channel, ChannelLocation}, restricted_string::RestrictedString, Event},
};

use super::{not_found, ok, unwrap, with_lock, with_login, ClientHolder, ResponseResult};

#[derive(Debug, Deserialize)]
pub struct ChannelCreate {
    name: RestrictedString,
    location: ChannelLocation,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChannelResponse {
    channel: Channel,
}

impl ChannelResponse {
    async fn from_channel(channel: Channel) -> Self {
        Self { channel }
    }
}

pub async fn create(
    token: String,
    create: ChannelCreate,
    clients: ClientHolder,
) -> ResponseResult<ChannelResponse> {
    let user = with_login!(token);

    let channel = unwrap!(Channel::new(create.name.clone(), create.location.clone()).insert().await);

    let event = Event::ChannelCreate {
        channel: channel.clone(),
        creator: user,
    };

    with_lock!(clients).dispatch_event(event);

    ok!(ChannelResponse::from_channel(channel).await)
}

pub async fn fetch(token: String, id: String) -> ResponseResult<ChannelResponse> {
    with_login!(token);

    let Some(channel) = unwrap!(database().await.fetch_channel(id.clone()).await) else {
        return not_found!("Channel")
    };

    ok!(ChannelResponse::from_channel(channel).await)
}
