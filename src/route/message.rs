// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use warp::hyper::StatusCode;

use crate::{
    database, map_async,
    structures::{message::Message, user::User, Event, channel::Channel},
};

use super::{
    not_found, ok, unwrap, with_lock, with_login, ClientHolder, HttpError, Response, ResponseResult,
};

#[derive(Debug, Deserialize)]
pub struct MessageCreate {
    channel_id: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    message: Message,
    channel: Channel,
    author: Option<User>,
}

impl MessageResponse {
    async fn from_message(message: Message) -> Self {
        let Some(id) = &message.author_id else {
            return Self::none(message).await;
        };

        let Some(user) = database().await.fetch_user(id.clone()).await.unwrap_or(None) else {
            return Self::none(message).await;
        };

        Self::from(message, Some(user)).await
    }

    async fn none(message: Message) -> Self {
        Self::from(message, None).await
    }

    async fn from(message: Message, author: Option<User>) -> Self {
        let channel = database().await.fetch_channel(message.channel_id.clone()).await.unwrap().unwrap();
        MessageResponse {
            message,
            channel,
            author,
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

    if let None = unwrap!(database().await.fetch_channel(create.channel_id.clone()).await) {
        return not_found!("Channel")
    };

    let message = unwrap!(
        Message::new(create.channel_id.clone(), user.id.clone(), create.content.clone())
            .insert()
            .await
    );

    let event = Event::MessageCreate {
        message: message.clone(),
        author: Some(user.clone()),
    };

    with_lock!(clients).dispatch_event(event);

    ok!(MessageResponse::from(message, Some(user)).await)
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
    channel: String,
    before: String,
    max: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct FetchAfter {
    channel: String,
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
            .fetch_messages_before(query.channel.clone(), query.before.clone(), max)
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
            .fetch_messages_after(query.channel.clone(), query.after.clone(), max)
            .await
    );

    ok!(map_async(messages, MessageResponse::from_message).await)
}
