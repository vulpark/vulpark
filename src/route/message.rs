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
    message: MessageCreate,
    clients: Clients,
) -> ResponseResult<Message> {
    let user = with_login!(token);

    let message = unwrap!(
        Message::from_user(user.id.clone(), message.content.clone())
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

pub async fn fetch(message_id: String, token: String) -> ResponseResult<Message> {
    with_login!(token);

    let Some(message) = unwrap!(database().await.fetch_message(message_id.clone()).await) else {
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
