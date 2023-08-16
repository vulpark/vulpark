// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod channel;
mod gateway;
mod guild;
mod macros;
mod message;
mod user;

use reqwest::StatusCode;
use rweb::openapi;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use warp::reject::{MethodNotAllowed, MissingHeader};
use warp::ws::MissingConnectionUpgrade;
use warp::{Filter, Rejection};

use crate::structures::client::{ClientHolder, Clients};
use crate::structures::error::{HttpError, ResponseResult};
use crate::structures::response::Response;

use self::macros::{err, not_found};

pub async fn init() {
    let clients: ClientHolder = Arc::new(Mutex::new(Clients(HashMap::new())));

    //TODO: Figure out what to do with spec
    let (_spec, filter) = openapi::spec().build(|| {
        gateway::routes(&clients)
            .or(message::routes(&clients))
            .or(channel::routes(&clients))
            .or(user::routes())
            .or(guild::routes())
            .recover(recover)
            .with(warp::cors().allow_any_origin())
    });

    warp::serve(filter).run(([127, 0, 0, 1], 8000)).await;
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
            StatusCode::BAD_REQUEST
        );
    }
    err!(
        HttpError::Other(format!(
            "Unhandled error: {:?}. Report to us at https://github.com/vulpark/vulpark/issues",
            rejection
        )),
        StatusCode::INTERNAL_SERVER_ERROR
    )
}
