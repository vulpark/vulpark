// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    database,
    structures::{
        channel::{Channel, ChannelCreate, ChannelResponse},
        error::ResponseResult,
        event::Event,
    },
    with_lock,
};

use super::{
    macros::{not_found, ok, unwrap, with_login},
    ClientHolder,
};

pub async fn create(
    token: String,
    create: ChannelCreate,
    clients: ClientHolder,
) -> ResponseResult<ChannelResponse> {
    let user = with_login!(token);

    let channel = unwrap!(
        Channel::new(&create.name, create.location.clone())
            .insert()
            .await
    );

    let event = Event::ChannelCreate {
        channel: channel.clone(),
        creator: user,
    };

    with_lock!(clients).dispatch_users(channel.get_users(), &event);

    ok!(ChannelResponse::from_channel(channel))
}

pub async fn fetch(token: String, id: String) -> ResponseResult<ChannelResponse> {
    with_login!(token);

    let Some(channel) = unwrap!(database().await.fetch_channel(id.clone()).await) else {
        return not_found!("Channel")
    };

    ok!(ChannelResponse::from_channel(channel))
}
