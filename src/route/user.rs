// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use warp::hyper::StatusCode;

use crate::{
    database,
    route::{expect, with_login, HttpError},
    structures::user::User,
};

use super::{unwrap, Response, ResponseResult};

#[derive(Debug, Deserialize)]
pub struct UserCreateRequest {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct UserCreateResponse {
    pub user: User,
    pub token: String,
}

impl From<(User, String)> for UserCreateResponse {
    fn from(value: (User, String)) -> Self {
        Self {
            user: value.0,
            token: value.1,
        }
    }
}

pub async fn create(user: UserCreateRequest) -> ResponseResult<UserCreateResponse> {
    let user = expect!(
        unwrap!(User::create(user.username.clone()).await),
        StatusCode::INTERNAL_SERVER_ERROR,
        HttpError::TooManyUsers
    );

    Ok(warp::reply::with_status(
        Response::success(user.into()),
        StatusCode::CREATED,
    ))
}

pub async fn fetch(user_id: String, token: String) -> ResponseResult<User> {
    with_login!(token);

    let user = expect!(
        unwrap!(database().await.fetch_user(user_id.clone()).await),
        StatusCode::INTERNAL_SERVER_ERROR,
        HttpError::TooManyUsers
    );

    Ok(warp::reply::with_status(
        Response::success(user.into()),
        StatusCode::OK,
    ))
}
