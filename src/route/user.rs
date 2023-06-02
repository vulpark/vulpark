use serde::Deserialize;
use warp::hyper::StatusCode;

use crate::{route::{Response, with_lock, with_client}, structures::Event};

use super::{Clients, ResponseResult};

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
}

pub async fn login(client_id: String, req: LoginRequest, clients: Clients) -> ResponseResult<()> {
    let mut lock = with_lock!(clients);
    let client = with_client!(lock, client_id);
    let username = Some(req.username.clone());
    client.username = username;
    client.send(&Event::LoginSuccess {
        username: req.username.clone(),
    });
    println!("Setting {}, {}", client_id, req.username.clone());
    Ok(warp::reply::with_status(
        Response::Success { data: () },
        StatusCode::OK,
    ))
}
