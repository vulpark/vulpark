// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use warp::{Filter, Rejection, Reply};

use crate::{
    route::macros::{ok, unwrap, with_login},
    structures::{
        error::ResponseResult,
        guild::{Guild, GuildCreate, GuildResponse},
    },
};

use super::with_auth;

pub fn routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let create = warp::post()
        .and(with_auth())
        .and(warp::body::json())
        .and_then(create);

    warp::path("guilds").and(create)
}

pub async fn create(token: String, create: GuildCreate) -> ResponseResult<GuildResponse> {
    let user = with_login!(token);

    let guild = unwrap!(Guild::new(&create.name, &user.id).insert().await);

    let resp = GuildResponse::new(guild, user);

    ok!(resp)
}
