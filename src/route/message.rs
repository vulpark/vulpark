use serde::Deserialize;
use warp::hyper::StatusCode;

use crate::{structures::{message::Message, Event}, database, route::HttpError};

use super::{Clients, Response, ResponseResult, with_lock, with_client, with_login, unwrap, check_login};

#[derive(Debug, Deserialize)]
pub struct MessageCreate {
    content: String,
}

pub async fn create(client_id: String, message: MessageCreate, clients: Clients) -> ResponseResult<Message> {
    let mut lock = with_lock!(clients);
    let client = with_client!(lock, client_id);
    let username = with_login!(client);

    let message = unwrap!(Message::new(username.clone(), message.content.clone()).insert().await);

    let event = Event::MessageCreate {
        message: message.clone(),
    };

    lock.values().for_each(|it| it.send(&event));

    Ok(warp::reply::with_status(
        Response::Success { data: message },
        StatusCode::CREATED,
    ))
}

pub async fn fetch(message_id: String, client_id: String, clients: Clients) -> ResponseResult<Message> {
    check_login!(clients, client_id);

    let Some(message) = unwrap!(database().await.fetch_message(message_id.clone()).await) else {
        return Ok(warp::reply::with_status(Response::Error {
            status_code: 404,
            message: HttpError::MessageNotFound
        }, StatusCode::NOT_FOUND))
    };

    Ok(warp::reply::with_status(
        Response::Success { data: message },
        StatusCode::CREATED,
    ))
}
