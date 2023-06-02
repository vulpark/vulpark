mod gateway;
mod message;
mod user;

use serde::Serialize;
use std::convert::Infallible;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use ulid::Ulid;
use warp::hyper::StatusCode;
use warp::reply::WithStatus;
use warp::ws::Message;
use warp::{Filter, Rejection, Reply};

use crate::structures::Event;

use self::gateway::gateway;

type ResponseResult<T> = Result<WithStatus<Response<T>>, Rejection>;

#[derive(Debug)]
pub enum HttpError {
    WebsocketNotConnected,
    InvalidLoginCredentials,
    MessageNotFound,
    Other(String),
}

impl ToString for HttpError {
    fn to_string(&self) -> String {
        match self {
            Self::WebsocketNotConnected => "Client ID not found.",
            Self::InvalidLoginCredentials => "Not logged in.",
            Self::MessageNotFound => "Message not found.",
            Self::Other(msg) => msg
        }.into()
    }
}

impl Serialize for HttpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize, Debug)]
pub enum Response<T>
where
    T: Serialize,
{
    Error { status_code: u16, message: HttpError },
    Success { data: T },
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
    pub username: Option<String>,
}

impl Client {
    pub fn empty() -> Self {
        Client {
            id: Ulid::new().to_string(),
            sender: None,
            username: None,
        }
    }

    pub fn send(&self, message: &Event) {
        let Some(ref sender) = self.sender else { return };
        let _ = sender.send(Ok(Message::text(message.to_string())));
    }

    pub fn get_name(&self) -> String {
        if let Some(username) = &self.username {
            return username.clone();
        }
        self.id.clone()
    }
}

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;

pub async fn init() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let gateway = warp::path("gateway")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(gateway);

    let login = warp::path("login")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(user::login);

    let message_create = warp::path("messages")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(message::create);

    let message_fetch = warp::path("messages")
        .and(warp::get())
        .and(warp::path::param())
        .and(with_auth())
        .and(with_clients(clients.clone()))
        .and_then(message::fetch);

    let routes = gateway
        .or(login)
        .or(message_create)
        .or(message_fetch)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_auth() -> impl warp::Filter<Extract = (String,), Error = Rejection> + Copy {
    warp::header("Authorization")
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

pub macro with_lock($clients: expr) {
    $clients.lock().await
}

pub macro with_client($lock: expr, $client_id: expr) {
    if let Some(client) = $lock.get_mut(&$client_id) {
        client
    } else {
        return Ok(warp::reply::with_status(Response::Error {
            status_code: 404,
            message: HttpError::WebsocketNotConnected
        }, StatusCode::NOT_FOUND))
    }
}

pub macro with_login($client: expr) {
    if let Some(username) = &$client.username {
        username
    } else {
        return Ok(warp::reply::with_status(Response::Error {
            status_code: 403,
            message: HttpError::InvalidLoginCredentials
        }, StatusCode::FORBIDDEN))
    }
}

pub macro check_login($clients: expr, $client_id: expr) {
    with_login!(with_client!(with_lock!($clients), $client_id));
}

pub macro unwrap($req: expr) {
    if let Ok(val) = $req {
        val
    } else {
        return Ok(warp::reply::with_status(Response::Error {
            status_code: 500,
            message: HttpError::Other($req.unwrap_err().to_string())
        }, StatusCode::INTERNAL_SERVER_ERROR));
    }
}
