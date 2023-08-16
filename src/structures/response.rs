// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use rweb::{Schema, openapi::{ResponseEntity, Entity, Responses}};
use serde::{ser::SerializeStruct, Serialize};
use warp::Reply;

use super::error::HttpError;

#[derive(Debug, Schema)]
pub enum Response<T>
where
    T: Serialize,
{
    Error {
        status_code: u16,
        message: HttpError,
    },
    Success {
        data: T,
    },
}

impl<T> Serialize for Response<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Error {
                status_code,
                message,
            } => {
                let mut err = serializer.serialize_struct("Error", 2)?;
                err.serialize_field("status_code", status_code)?;
                err.serialize_field("message", message)?;
                err.end()
            }
            Self::Success { data } => data.serialize(serializer),
        }
    }
}

impl<T> Response<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Response<T> {
        Self::Success { data }
    }
}

impl<T> ToString for Response<T>
where
    T: Serialize,
{
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl<T> Reply for Response<T>
where
    T: Serialize + std::marker::Send,
{
    fn into_response(self) -> warp::reply::Response {
        self.to_string().into_response()
    }
}

impl <T: Entity + Serialize> ResponseEntity for Response<T> {
    fn describe_responses(comp_d: &mut rweb::openapi::ComponentDescriptor) -> Responses {
        let resp = Responses::new();
        resp
    }
}
