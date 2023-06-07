// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::Serialize;
use warp::{reply::WithStatus, Rejection};

use super::response::Response;

pub type ResponseResult<T> = Result<WithStatus<Response<T>>, Rejection>;

#[derive(Debug)]
pub enum HttpError {
    InvalidLoginCredentials,
    NotFound(String),
    MessageContentEmpty,
    ChannelAccessDenied,
    TooManyUsers,
    Other(String),
}

impl ToString for HttpError {
    fn to_string(&self) -> String {
        match self {
            Self::InvalidLoginCredentials => "Invalid login credentials.".to_string(),
            Self::NotFound(name) => format!("{name} not found."),
            Self::MessageContentEmpty => "Message content is empty.".to_string(),
            Self::ChannelAccessDenied => "Channel access is denied".to_string(),
            Self::TooManyUsers => "Too many users with the same username".to_string(),
            Self::Other(msg) => msg.to_string(),
        }
    }
}

impl Serialize for HttpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
