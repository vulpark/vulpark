// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
    with_auth, with_clients, ClientHolder,
};

pub fn routes(
    clients: &ClientHolder,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let create = warp::post()
        .and(with_auth())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(create);

    let fetch = warp::get()
        .and(with_auth())
        .and(warp::path::param())
        .and_then(fetch);

    warp::path("channels").and(create.or(fetch))
}

pub async fn create(
    token: String,
    create: ChannelCreate,
    clients: ClientHolder,
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

pub async fn fetch(token: String, id: String) -> ResponseResult<ChannelResponse> {
    with_login!(token);

    let Some(channel) = unwrap!(database().await.fetch_channel(id.clone()).await) else {
        return not_found!("Channel")
    };

    ok!(ChannelResponse::from_channel(channel))
}
