// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use warp::hyper::StatusCode;

use crate::{
    database,
    structures::{
        error::ResponseResult,
        user::{User, UserCreateRequest, UserCreateResponse},
    },
};

use super::{
    macros::{expect, not_found, ok, unwrap, with_login},
    HttpError,
};

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

    ok!(user.into())
}

pub async fn fetch(user_id: String, token: String) -> ResponseResult<User> {
    with_login!(token);

    let Some(user) = unwrap!(database().await.fetch_user(user_id.clone()).await) else {
        return not_found!("User")
    };

    ok!(user)
}
