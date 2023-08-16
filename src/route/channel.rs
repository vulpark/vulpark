// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use rweb::*;
use warp::{Filter, Rejection, Reply};

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

pub fn routes(
    clients: &ClientHolder,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let create = create(clients.clone());

    let fetch = fetch();

    create.or(fetch)
}

#[post("/channels")]
pub async fn create(
    #[header = "Authentication"] token: String,
    #[json] create: ChannelCreate,
    #[data] clients: ClientHolder,
) -> ResponseResult<ChannelResponse> {
    with_login!(token);

    let channel = unwrap!(
        Channel::new(&create.name, create.location.clone())
            .insert()
            .await
    );

    let resp = ChannelResponse::from_channel(channel);

    with_lock!(clients).dispatch_users(
        resp.channel.get_users().await.unwrap_or(vec![]),
        &Event::ChannelCreate(resp.clone()),
    );

    ok!(resp)
}

#[get("/channels/{id}")]
pub async fn fetch(
    #[header = "Authentication"]  token: String,
    id: String
) -> ResponseResult<ChannelResponse> {
    with_login!(token);

    let Some(channel) = unwrap!(database().await.fetch_channel(id.clone()).await) else {
        return not_found!("Channel")
    };

    ok!(ChannelResponse::from_channel(channel))
}
