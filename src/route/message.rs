use serde::Deserialize;
use warp::hyper::StatusCode;

use crate::{
    database,
    route::HttpError,
    structures::{message::Message, Event},
};

use super::{unwrap, with_lock, with_login, Clients, Response, ResponseResult};

#[derive(Debug, Deserialize)]
pub struct MessageCreate {
    content: String,
}

pub async fn create(
    token: String,
    create: MessageCreate,
    clients: Clients,
) -> ResponseResult<Message> {
    let user = with_login!(token);

    if create.content.is_empty() {
        return Ok(warp::reply::with_status(Response::Error {
            status_code: 400,
            message: HttpError::MessageContentEmpty
        }, StatusCode::BAD_REQUEST))
    }

    let message = unwrap!(
        Message::from_user(user.id.clone(), create.content.clone())
            .insert()
            .await
    );

    let event = Event::MessageCreate {
        message: message.clone(),
        author: Some(user),
    };

    with_lock!(clients).values().for_each(|it| it.send(&event));

    Ok(warp::reply::with_status(
        Response::success(message),
        StatusCode::CREATED,
    ))
}

pub async fn fetch_single(token: String, id: String) -> ResponseResult<Message> {
    with_login!(token);

    let Some(message) = unwrap!(database().await.fetch_message(id.clone()).await) else {
        return Ok(warp::reply::with_status(Response::Error {
            status_code: 404,
            message: HttpError::MessageNotFound
        }, StatusCode::NOT_FOUND))
    };

    Ok(warp::reply::with_status(
        Response::success(message),
        StatusCode::OK,
    ))
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

pub async fn fetch_before(token: String, query: FetchBefore) -> ResponseResult<Vec<Message>> {
    with_login!(token);

    let max = query.max.unwrap_or(25).min(25);

    let messages = unwrap!(
        database()
            .await
            .fetch_messages_before(query.before.clone(), max)
            .await
    );

    Ok(warp::reply::with_status(
        Response::success(messages),
        StatusCode::OK,
    ))
}

pub async fn fetch_after(token: String, query: FetchAfter) -> ResponseResult<Vec<Message>> {
    with_login!(token);

    let max = query.max.unwrap_or(25).min(25);

    let messages = unwrap!(
        database()
            .await
            .fetch_messages_after(query.after.clone(), max)
            .await
    );

    Ok(warp::reply::with_status(
        Response::success(messages),
        StatusCode::OK,
    ))
}
