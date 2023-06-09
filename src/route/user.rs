// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use warp::hyper::StatusCode;

use crate::{
    database,
    structures::{
        auth::{Login, AuthError},
        error::ResponseResult,
        user::{User, UserCreateRequest, UserLoginResponse, UserLoginRequest},
    },
};

use super::{
    macros::{err, expect, not_found, ok, unwrap, with_login},
    HttpError,
};

impl From<(User, String)> for UserLoginResponse {
    fn from(value: (User, String)) -> Self {
        Self {
            user: value.0,
            token: value.1,
        }
    }
}

pub async fn create(user: UserCreateRequest) -> ResponseResult<UserLoginResponse> {
    let service = user.service;

    let token = match service.fetch_token(&user.oauth_code).await {
        Result::Ok(token) => token,
        Result::Err(err) => return err!(HttpError::Oauth(err), StatusCode::INTERNAL_SERVER_ERROR),
    };

    let uid = match service.get_uid(token).await {
        Result::Ok(token) => token,
        Result::Err(err) => return err!(HttpError::Oauth(err), StatusCode::INTERNAL_SERVER_ERROR),
    };

    match database().await.fetch_login(service, uid.clone()).await {
        Ok(login) => match login {
            Some(_) => return err!(HttpError::AccountAttached, StatusCode::FORBIDDEN),
            None => {}
        },
        Err(error) => return err!(HttpError::Oauth(AuthError::Mongo(error)), StatusCode::INTERNAL_SERVER_ERROR)
    }

    let user = expect!(
        unwrap!(User::create(&user.username).await),
        StatusCode::INTERNAL_SERVER_ERROR,
        HttpError::TooManyUsers
    );

    let login = Login::new(service, uid, user.0.id.clone());

    let _ = database().await.create_login(login).await;

    ok!(user.into())
}

pub async fn login(user: UserLoginRequest) -> ResponseResult<UserLoginResponse> {
    let service = user.service;

    let token = match service.fetch_token(&user.oauth_code).await {
        Result::Ok(token) => token,
        Result::Err(err) => return err!(HttpError::Oauth(err), StatusCode::INTERNAL_SERVER_ERROR),
    };

    let uid = match service.get_uid(token).await {
        Result::Ok(token) => token,
        Result::Err(err) => return err!(HttpError::Oauth(err), StatusCode::INTERNAL_SERVER_ERROR),
    };

    let login = match database().await.fetch_login(service, uid.clone()).await {
        Ok(login) => match login {
            Some(login) => login,
            None => return err!(HttpError::AccountNotAttached, StatusCode::FORBIDDEN)
        },
        Err(error) => return err!(HttpError::Oauth(AuthError::Mongo(error)), StatusCode::INTERNAL_SERVER_ERROR)
    };

    let Some(user) = unwrap!(database().await.fetch_user_login(login.user_id.clone()).await) else {
        return not_found!("User")
    };

    ok!(user.into())
}

pub async fn fetch(user_id: String, token: String) -> ResponseResult<User> {
    with_login!(token);

    let Some(user) = unwrap!(database().await.fetch_user(user_id.clone()).await) else {
        return not_found!("User")
    };

    ok!(user)
}
