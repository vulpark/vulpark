// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use rweb::*;
use warp::{hyper::StatusCode, Filter, Rejection, Reply};

use crate::{
    database, map_async,
    structures::{
        error::ResponseResult,
        event::Event,
        message::{Message, MessageCreate, MessageFetchAfter, MessageFetchBefore, MessageResponse},
    },
    with_lock,
};

use super::{
    macros::{err, not_found, ok, unwrap, with_login},
    ClientHolder, HttpError,
};

pub fn routes(
    clients: &ClientHolder,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    create(clients.clone()).or(fetch_single()).or(fetch_before()).or(fetch_after())
}

#[post("/messages")]
pub async fn create(
    #[header = "Authentication"] token: String,
    #[json] create: MessageCreate,
    #[data] clients: ClientHolder,
) -> ResponseResult<MessageResponse> {
    let user = with_login!(token);

    if create.content.is_empty() {
        return err!(HttpError::MessageContentEmpty, StatusCode::BAD_REQUEST);
    }

    let Some(channel) = unwrap!(database().await.fetch_channel(create.channel_id.clone()).await) else {
        return not_found!("Channel")
    };

    let users = channel.get_users().await.unwrap_or(vec![]);

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

    let resp = MessageResponse::from(message, channel, Some(user));

    with_lock!(clients).dispatch_users(users, &Event::MessageCreate(resp.clone()));

    ok!(resp)
}

#[get("/messages/{id}")]
pub async fn fetch_single(
    #[header = "Authentication"] token: String,
    id: String
) -> ResponseResult<MessageResponse> {
    let user = with_login!(token);

    let Some(message) = unwrap!(database().await.fetch_message(id.clone()).await) else {
        return not_found!("Message")
    };

    let Some(channel) = unwrap!(database().await.fetch_channel(message.channel_id.clone()).await) else {
        return not_found!("Channel")
    };

    let users = channel.get_users().await.unwrap_or(vec![]);

    if !users.contains(&user.id) {
        return err!(HttpError::ChannelAccessDenied, StatusCode::FORBIDDEN);
    }

    ok!(MessageResponse::from_message(message, channel).await)
}

#[get("/messages")]
pub async fn fetch_before(
    #[header = "Authentication"] token: String,
    #[filter = "warp::query"] query: MessageFetchBefore,
) -> ResponseResult<Vec<MessageResponse>> {
    let user = with_login!(token);

    let Some(channel) = unwrap!(database().await.fetch_channel(query.channel.clone()).await) else {
        return not_found!("Channel")
    };

    let users = channel.get_users().await.unwrap_or(vec![]);

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

#[get("/messages")]
pub async fn fetch_after(
    #[header = "Authentication"] token: String,
    #[filter = "warp::query"] query: MessageFetchAfter,
) -> ResponseResult<Vec<MessageResponse>> {
    let user = with_login!(token);

    let Some(channel) = unwrap!(database().await.fetch_channel(query.channel.clone()).await) else {
        return not_found!("Channel")
    };

    let users = channel.get_users().await.unwrap_or(vec![]);

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
