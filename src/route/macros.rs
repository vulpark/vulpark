// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use reqwest::StatusCode;

use crate::database;

use super::{HttpError, Response};

pub macro ok($data: expr) {
    Ok(warp::reply::with_status(
        Response::success($data),
        StatusCode::OK,
    ))
}

pub macro err($message: expr, $status: expr) {
    Ok(warp::reply::with_status(
        Response::Error {
            status_code: $status.as_u16(),
            message: $message,
        },
        $status,
    ))
}

pub macro not_found($name: expr) {
    err!(
        HttpError::NotFound($name.to_string()),
        StatusCode::NOT_FOUND
    )
}

pub macro with_login($token: expr) {
    expect!(
        unwrap!(database().await.fetch_user_token($token.clone()).await),
        StatusCode::FORBIDDEN,
        HttpError::InvalidLoginCredentials
    )
}

pub macro unwrap($req: expr) {
    if let Ok(val) = $req {
        val
    } else {
        return err!(
            HttpError::Other($req.unwrap_err().to_string()),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}

pub macro expect($val: expr, $status: expr, $message: expr) {
    if let Some(val) = $val {
        val
    } else {
        return err!($message, $status);
    }
}
