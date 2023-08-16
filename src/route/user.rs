// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use rweb::*;
use warp::{hyper::StatusCode, Filter, Rejection, Reply};

use crate::{
    database,
    structures::{
        auth::Login,
        error::ResponseResult,
        user::{User, UserCreateRequest, UserLoginRequest, UserLoginResponse},
    },
};

use super::{
    macros::{err, expect, not_found, ok, unwrap, with_login},
    HttpError,
};

pub fn routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    create().or(fetch()).or(login())
}

#[post("/users")]
pub async fn create(
    #[json] user: UserCreateRequest
) -> ResponseResult<UserLoginResponse> {
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
        Ok(login) => {
            if login.is_some() {
                return err!(HttpError::AccountAttached, StatusCode::FORBIDDEN);
            }
        }
        Err(error) => {
            return err!(
                HttpError::Oauth(error.into()),
                StatusCode::INTERNAL_SERVER_ERROR
            )
        }
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

#[get("/users")]
pub async fn login(
    #[json] user: UserLoginRequest
) -> ResponseResult<UserLoginResponse> {
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
            None => return err!(HttpError::AccountNotAttached, StatusCode::FORBIDDEN),
        },
        Err(error) => {
            return err!(
                HttpError::Oauth(error.into()),
                StatusCode::INTERNAL_SERVER_ERROR
            )
        }
    };

    let Some(user) = unwrap!(database().await.fetch_user_login(&login.user_id).await) else {
        return not_found!("User")
    };

    ok!(user.into())
}

#[get("/users/{id}")]
pub async fn fetch(
    #[header = "Authorization"] token: String,
    id: String,
) -> ResponseResult<User> {
    with_login!(token);

    let Some(user) = unwrap!(database().await.fetch_user(&id).await) else {
        return not_found!("User")
    };

    ok!(user)
}
