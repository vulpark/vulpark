// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod auth;
mod channel;
mod gateway;
mod message;
mod user;

use serde::ser::SerializeStruct;
use serde::Serialize;
use std::convert::Infallible;
use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use ulid::Ulid;
use warp::hyper::StatusCode;
use warp::reject::{MethodNotAllowed, MissingHeader};
use warp::reply::WithStatus;
use warp::ws::{Message, MissingConnectionUpgrade};
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

#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
    pub user_id: Option<String>,
}

impl Client {
    pub fn empty() -> Self {
        Client {
            id: Ulid::new().to_string(),
            sender: None,
            user_id: None,
        }
    }

    pub fn send(&self, event: &Event) {
        let Some(ref sender) = self.sender else { return };
        let _ = sender.send(Ok(Message::text(event.to_string())));
    }

    pub async fn set_user(&mut self, token: String) -> Option<User> {
        let user = database().await.fetch_user_token(token.clone()).await;
        let Ok(user) = user else {
            return None;
        };
        let user = user?;

        self.user_id = Some(user.id.clone());
        Some(user)
    }

    pub async fn remove_from(&self, clients: ClientHolder) -> Option<()> {
        let mut lock = with_lock!(clients);
        let id = self.user_id.clone()?;
        let clients = lock.get_mut(&id)?;
        let index = clients.into_iter().position(|it| it.id == self.id)?;
        clients.remove(index);
        Some(())
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

pub struct Clients(HashMap<String, Vec<Client>>);

impl Clients {
    pub fn dispatch_global(&self, event: Event) {
        self.values()
            .for_each(|clients| Self::dispatch_to(clients, &event));
    }

    pub fn dispatch_users(&self, users: Vec<String>, event: Event) {
        users.into_iter().for_each(|user| {
            if let Some(clients) = self.get(&user) {
                Self::dispatch_to(clients, &event)
            }
        })
    }

    fn dispatch_to(clients: &Vec<Client>, event: &Event) {
        clients.into_iter().for_each(|client| client.send(event))
    }
}

impl Deref for Clients {
    type Target = HashMap<String, Vec<Client>>;

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
        .recover(recover)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

async fn recover(rejection: Rejection) -> ResponseResult<()> {
    if rejection.is_not_found() {
        return not_found!("Route");
    }
    if let Some(header) = rejection.find::<MissingHeader>() {
        return err!(
            HttpError::Other(format!("Missing header: {}", header.name())),
            StatusCode::BAD_REQUEST
        );
    }
    if let Some(_) = rejection.find::<MethodNotAllowed>() {
        return err!(
            HttpError::Other("Method not allowed".to_string()),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
    if let Some(_) = rejection.find::<MissingConnectionUpgrade>() {
        return err!(
            HttpError::Other("Missing websocket upgrade".to_string()),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
    Err(rejection)
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

pub macro ok($data: expr) {
    Ok(warp::reply::with_status(
        Response::success($data),
        StatusCode::OK,
    ))
}

pub macro err($message: expr, $status: expr) {
    Ok(warp::reply::with_status(
        Response::Error {
            status_code: $status.as_u16(),
            message: $message,
        },
        $status,
    ))
}

pub macro not_found($name: expr) {
    err!(
        HttpError::NotFound($name.to_string()),
        StatusCode::NOT_FOUND
    )
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
        return err!(
            HttpError::Other($req.unwrap_err().to_string()),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}

pub macro expect($val: expr, $status: expr, $message: expr) {
    if let Some(val) = $val {
        val
    } else {
        return err!($message, $status);
    }
}
