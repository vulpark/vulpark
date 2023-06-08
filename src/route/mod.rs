// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod auth;
mod channel;
mod gateway;
mod macros;
mod message;
mod user;

use reqwest::StatusCode;
use std::convert::Infallible;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use warp::reject::{MethodNotAllowed, MissingHeader};
use warp::ws::MissingConnectionUpgrade;
use warp::{Filter, Rejection};

use crate::structures::client::{ClientHolder, Clients};
use crate::structures::error::{HttpError, ResponseResult};
use crate::structures::response::Response;

use self::gateway::gateway;
use self::macros::{err, not_found};

pub async fn init() {
    let clients: ClientHolder = Arc::new(Mutex::new(Clients(HashMap::new())));

    let gateway = warp::path("gateway")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(gateway);

    let message_create = warp::path("messages")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(message::create);

    let message_fetch_single = warp::path("messages")
        .and(warp::get())
        .and(with_auth())
        .and(warp::path::param())
        .and_then(message::fetch_single);

    let message_fetch_before = warp::path("messages")
        .and(warp::get())
        .and(with_auth())
        .and(warp::query())
        .and_then(message::fetch_before);

    let message_fetch_after = warp::path("messages")
        .and(warp::get())
        .and(with_auth())
        .and(warp::query())
        .and_then(message::fetch_after);

    let channel_create = warp::path("channels")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(channel::create);

    let channel_fetch = warp::path("messages")
        .and(warp::get())
        .and(with_auth())
        .and(warp::path::param())
        .and_then(channel::fetch);

    let user_create = warp::path("users")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(user::create);

    let user_fetch = warp::path("users")
        .and(warp::get())
        .and(warp::path::param())
        .and(with_auth())
        .and_then(user::fetch);

    let routes = gateway
        .or(message_create)
        .or(message_fetch_single)
        .or(message_fetch_before)
        .or(message_fetch_after)
        .or(channel_create)
        .or(channel_fetch)
        .or(user_create)
        .or(user_fetch)
        .recover(recover)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

async fn recover(rejection: Rejection) -> ResponseResult<()> {
    if rejection.is_not_found() {
        return not_found!("Route");
    }
    if let Some(header) = rejection.find::<MissingHeader>() {
        return err!(
            HttpError::Other(format!("Missing header: {}", header.name())),
            StatusCode::BAD_REQUEST
        );
    }
    if rejection.find::<MethodNotAllowed>().is_some() {
        return err!(
            HttpError::Other("Method not allowed".to_string()),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
    if rejection.find::<MissingConnectionUpgrade>().is_some() {
        return err!(
            HttpError::Other("Missing websocket upgrade".to_string()),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
    err!(
        HttpError::Other(format!("{:?}", rejection)),
        StatusCode::INTERNAL_SERVER_ERROR
    )
}

fn with_auth() -> impl warp::Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::header("Authorization")
}

fn with_clients(
    clients: ClientHolder,
) -> impl Filter<Extract = (ClientHolder,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
