// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod channel;
mod gateway;
mod message;
mod user;

use serde::Serialize;
use std::convert::Infallible;
use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use ulid::Ulid;
use warp::hyper::StatusCode;
use warp::reply::WithStatus;
use warp::ws::Message;
use warp::{Filter, Rejection, Reply};

use crate::database;
use crate::structures::user::User;
use crate::structures::Event;

use self::gateway::gateway;

type ResponseResult<T> = Result<WithStatus<Response<T>>, Rejection>;

#[derive(Debug)]
pub enum HttpError {
    InvalidLoginCredentials,
    NotFound(String),
    MessageContentEmpty,
    TooManyUsers,
    Other(String),
}

impl ToString for HttpError {
    fn to_string(&self) -> String {
        match self {
            Self::InvalidLoginCredentials => "Invalid login credentials.".to_string(),
            Self::NotFound(name) => format!("{name} not found."),
            Self::MessageContentEmpty => "Message content is empty.".to_string(),
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

#[derive(Serialize, Debug)]
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

#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
    pub token: Option<String>,
}

impl Client {
    pub fn empty() -> Self {
        Client {
            id: Ulid::new().to_string(),
            sender: None,
            token: None,
        }
    }

    pub fn send(&self, message: &Event) {
        let Some(ref sender) = self.sender else { return };
        let _ = sender.send(Ok(Message::text(message.to_string())));
    }

    pub async fn set_user(&mut self, token: String) -> Option<User> {
        let user = database().await.fetch_user_token(token.clone()).await;
        let Ok(user) = user else {
            return None;
        };
        self.token = Some(token);
        user
    }

    // pub async fn get_user(&self) -> Option<User> {
    //     let Some(token) = &self.token else {
    //         return None;
    //     };
    //     let user = database().await.fetch_user_token(token.clone()).await;
    //     let Ok(user) = user else {
    //         return None;
    //     };
    //     user
    // }
}

pub struct Clients(HashMap<String, Client>);

impl Clients {
    pub fn dispatch_event(&self, event: Event) {
        self.values().for_each(|client| client.send(&event))
    }
}

impl Deref for Clients {
    type Target = HashMap<String, Client>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Clients {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type ClientHolder = Arc<Mutex<Clients>>;

pub async fn init() {
    let clients: ClientHolder = Arc::new(Mutex::new(Clients(HashMap::new())));

    let gateway = warp::path("gateway")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(gateway);

    let message_create = warp::path("messages")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(message::create);

    let message_fetch_single = warp::path("messages")
        .and(warp::get())
        .and(with_auth())
        .and(warp::path::param())
        .and_then(message::fetch_single);

    let message_fetch_before = warp::path("messages")
        .and(warp::get())
        .and(with_auth())
        .and(warp::query())
        .and_then(message::fetch_before);

    let message_fetch_after = warp::path("messages")
        .and(warp::get())
        .and(with_auth())
        .and(warp::query())
        .and_then(message::fetch_after);

    let channel_create = warp::path("channels")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(channel::create);

    let channel_fetch = warp::path("messages")
        .and(warp::get())
        .and(with_auth())
        .and(warp::path::param())
        .and_then(channel::fetch);

    let user_create = warp::path("users")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(user::create);

    let user_fetch = warp::path("users")
        .and(warp::get())
        .and(warp::path::param())
        .and(with_auth())
        .and_then(user::fetch);

    let routes = gateway
        .or(message_create)
        .or(message_fetch_single)
        .or(message_fetch_before)
        .or(message_fetch_after)
        .or(channel_create)
        .or(channel_fetch)
        .or(user_create)
        .or(user_fetch)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_auth() -> impl warp::Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::header("Authorization")
}

fn with_clients(
    clients: ClientHolder,
) -> impl Filter<Extract = (ClientHolder,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

pub macro with_lock($clients: expr) {
    $clients.lock().await
}

pub macro not_found($name: expr) {
    Ok(warp::reply::with_status(
        Response::Error {
            status_code: 404,
            message: HttpError::NotFound($name.to_string()),
        },
        StatusCode::NOT_FOUND,
    ))
}

pub macro ok($data: expr) {
    Ok(warp::reply::with_status(
        Response::success($data),
        StatusCode::OK,
    ))
}

pub macro with_login($token: expr) {
    expect!(
        unwrap!(database().await.fetch_user_token($token.clone()).await),
        StatusCode::FORBIDDEN,
        HttpError::InvalidLoginCredentials
    )
}

pub macro unwrap($req: expr) {
    if let Ok(val) = $req {
        val
    } else {
        return Ok(warp::reply::with_status(
            Response::Error {
                status_code: 500,
                message: HttpError::Other($req.unwrap_err().to_string()),
            },
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
}

pub macro expect($val: expr, $status: expr, $message: expr) {
    if let Some(val) = $val {
        val
    } else {
        return Ok(warp::reply::with_status(
            Response::Error {
                status_code: $status.as_u16(),
                message: $message,
            },
            $status,
        ));
    }
}
