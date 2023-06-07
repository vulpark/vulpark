// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use warp::hyper::StatusCode;

use crate::{
    database, map_async,
    structures::{
        error::ResponseResult,
        event::Event,
        message::{Message, MessageCreate, MessageFetchAfter, MessageFetchBefore, MessageResponse},
    },
    with_lock,
};

use super::{macros::*, ClientHolder, HttpError};

pub async fn create(
    token: String,
    create: MessageCreate,
    clients: ClientHolder,
) -> ResponseResult<MessageResponse> {
    let user = with_login!(token);

    if create.content.is_empty() {
        return err!(HttpError::MessageContentEmpty, StatusCode::BAD_REQUEST);
    }

    let Some(channel) = unwrap!(database().await.fetch_channel(create.channel_id.clone()).await) else {
        return not_found!("Channel")
    };

    let users = channel.get_users().await;

    if !users.contains(&user.id) {
        return err!(HttpError::ChannelAccessDenied, StatusCode::FORBIDDEN);
    }

    let message = unwrap!(
        Message::new(
            create.channel_id.clone(),
            user.id.clone(),
            create.content.clone()
        )
        .insert()
        .await
    );

    let event = Event::MessageCreate {
        message: message.clone(),
        author: Some(user.clone()),
        channel: channel.clone(),
    };

    with_lock!(clients).dispatch_users(users, event);

    ok!(MessageResponse::from(message, channel, Some(user)).await)
}

pub async fn fetch_single(token: String, id: String) -> ResponseResult<MessageResponse> {
    let user = with_login!(token);

    let Some(message) = unwrap!(database().await.fetch_message(id.clone()).await) else {
        return not_found!("Message")
    };

    let Some(channel) = unwrap!(database().await.fetch_channel(message.channel_id.clone()).await) else {
        return not_found!("Channel")
    };

    let users = channel.get_users().await;

    if !users.contains(&user.id) {
        return err!(HttpError::ChannelAccessDenied, StatusCode::FORBIDDEN);
    }

    ok!(MessageResponse::from_message(message, channel).await)
}

pub async fn fetch_before(
    token: String,
    query: MessageFetchBefore,
) -> ResponseResult<Vec<MessageResponse>> {
    let user = with_login!(token);

    let Some(channel) = unwrap!(database().await.fetch_channel(query.channel.clone()).await) else {
        return not_found!("Channel")
    };

    let users = channel.get_users().await;

    if !users.contains(&user.id) {
        return err!(HttpError::ChannelAccessDenied, StatusCode::FORBIDDEN);
    }

    let max = query.max.unwrap_or(25).min(25);

    let messages = unwrap!(
        database()
            .await
            .fetch_messages_before(query.channel.clone(), query.before.clone(), max)
            .await
    );

    let channel = &channel;

    let mut out = vec![];
    map_async!(messages, out, |it| MessageResponse::from_message(
        it,
        channel.clone()
    ));

    ok!(out)
}

pub async fn fetch_after(
    token: String,
    query: MessageFetchAfter,
) -> ResponseResult<Vec<MessageResponse>> {
    let user = with_login!(token);

    let Some(channel) = unwrap!(database().await.fetch_channel(query.channel.clone()).await) else {
        return not_found!("Channel")
    };

    let users = channel.get_users().await;

    if !users.contains(&user.id) {
        return err!(HttpError::ChannelAccessDenied, StatusCode::FORBIDDEN);
    }

    let max = query.max.unwrap_or(25).min(25);

    let messages = unwrap!(
        database()
            .await
            .fetch_messages_after(query.channel.clone(), query.after.clone(), max)
            .await
    );

    let channel = &channel;

    let mut out = vec![];
    map_async!(messages, out, |it| MessageResponse::from_message(
        it,
        channel.clone()
    ));

    ok!(out)
}
