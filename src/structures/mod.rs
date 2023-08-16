// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use reqwest::StatusCode;
use rweb::{Schema, openapi::{ResponseEntity, Entity}};
use warp::{reply::Response, Reply};

pub mod auth;
pub mod channel;
pub mod client;
pub mod error;
pub mod event;
pub mod guild;
pub mod message;
pub mod response;
pub mod restricted_string;
pub mod user;

#[derive(Debug, Schema)]
pub struct WithStatus<T: Entity> {
    reply: T,
    status: u16,
}

impl<T: Reply + Entity> Reply for WithStatus<T> {
    fn into_response(self) -> Response {
        let mut res = self.reply.into_response();
        *res.status_mut() = StatusCode::from_u16(self.status).unwrap();
        res
    }
}

pub fn with_status<T: Entity>(data: T, status: StatusCode) -> WithStatus<T> {
    WithStatus { reply: data, status: status.as_u16() }
}

impl <T: Entity + ResponseEntity> ResponseEntity for WithStatus<T> {
    fn describe_responses(comp_d: &mut rweb::openapi::ComponentDescriptor) -> rweb::openapi::Responses {
        T::describe_responses(comp_d)
    }
}
