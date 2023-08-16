// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use rweb::*;
use warp::{Filter, Rejection, Reply};

use crate::{
    database,
    route::macros::{ok, unwrap, with_login},
    structures::{
        error::ResponseResult,
        guild::{Guild, GuildCreate, GuildResponse},
    },
};

pub fn routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let create = create();

    let fetch_all = fetch_all();

    create.or(fetch_all)
}

#[post("/guilds")]
pub async fn create(
    #[header = "Authentication"] token: String,
    #[json] create: GuildCreate
) -> ResponseResult<GuildResponse> {
    let user = with_login!(token);

    let guild = unwrap!(Guild::new(&create.name, &user.id).insert().await);

    let resp = GuildResponse::new(guild, user);

    ok!(resp)
}

#[get("/guilds")]
pub async fn fetch_all(
    #[header = "Authentication"] token: String
) -> ResponseResult<Vec<GuildResponse>> {
    let user = with_login!(token);

    let guilds = unwrap!(database().await.fetch_guilds_from_user(&user.id).await)
        .option()
        .unwrap();

    let mut resp = vec![];

    for guild in guilds {
        if let Ok(guild) = GuildResponse::from(guild).await {
            resp.push(guild);
        }
    }

    ok!(resp)
}
