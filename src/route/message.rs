// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use warp::hyper::StatusCode;

use crate::{
    database, map_async,
    structures::{message::Message, user::User, Event},
};

use super::{
    not_found, ok, unwrap, with_lock, with_login, ClientHolder, HttpError, Response, ResponseResult,
};

#[derive(Debug, Deserialize)]
pub struct MessageCreate {
    content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    message: Message,
    author: Option<User>,
}

impl MessageResponse {
    async fn from_message(message: Message) -> Self {
        let Some(id) = &message.author_id else {
            return Self::none(message);
        };

        let Some(user) = database().await.fetch_user(id.clone()).await.unwrap_or(None) else {
            return Self::none(message);
        };

        Self::from(message, user)
    }

    fn none(message: Message) -> Self {
        MessageResponse {
            message,
            author: None,
        }
    }

    fn from(message: Message, author: User) -> Self {
        MessageResponse {
            message,
            author: Some(author),
        }
    }
}

pub async fn create(
    token: String,
    create: MessageCreate,
    clients: ClientHolder,
) -> ResponseResult<MessageResponse> {
    let user = with_login!(token);

    if create.content.is_empty() {
        return Ok(warp::reply::with_status(
            Response::Error {
                status_code: 400,
                message: HttpError::MessageContentEmpty,
            },
            StatusCode::BAD_REQUEST,
        ));
    }

    let message = unwrap!(
        Message::from_user(user.id.clone(), create.content.clone())
            .insert()
            .await
    );

    let event = Event::MessageCreate {
        message: message.clone(),
        author: Some(user.clone()),
    };

    with_lock!(clients).dispatch_event(event);

    ok!(MessageResponse::from(message, user))
}

pub async fn fetch_single(token: String, id: String) -> ResponseResult<MessageResponse> {
    with_login!(token);

    let Some(message) = unwrap!(database().await.fetch_message(id.clone()).await) else {
        return not_found!("Message")
    };

    ok!(MessageResponse::from_message(message).await)
}

#[derive(Debug, Deserialize)]
pub struct FetchBefore {
    before: String,
    max: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct FetchAfter {
    after: String,
    max: Option<i64>,
}

pub async fn fetch_before(
    token: String,
    query: FetchBefore,
) -> ResponseResult<Vec<MessageResponse>> {
    with_login!(token);

    let max = query.max.unwrap_or(25).min(25);

    let messages = unwrap!(
        database()
            .await
            .fetch_messages_before(query.before.clone(), max)
            .await
    );

    ok!(map_async(messages, MessageResponse::from_message).await)
}

pub async fn fetch_after(token: String, query: FetchAfter) -> ResponseResult<Vec<MessageResponse>> {
    with_login!(token);

    let max = query.max.unwrap_or(25).min(25);

    let messages = unwrap!(
        database()
            .await
            .fetch_messages_after(query.after.clone(), max)
            .await
    );

    ok!(map_async(messages, MessageResponse::from_message).await)
}
