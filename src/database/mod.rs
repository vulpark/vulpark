// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::used_underscore_binding)]

mod auth;
mod channel;
mod macros;
mod message;
mod user;

use futures::stream::TryStreamExt;
use futures::TryStream;
use mongodb::Cursor;
use mongodb::{error::Result, options::ClientOptions, Client, Collection};

use self::auth::DatabaseLogin;
use self::channel::DatabaseChannel;
use self::message::DatabaseMessage;
use self::user::DatabaseUser;

#[allow(dead_code)]
pub struct Database {
    client: Client,
    db: mongodb::Database,
    messages: Collection<DatabaseMessage>,
    channels: Collection<DatabaseChannel>,
    users: Collection<DatabaseUser>,
    logins: Collection<DatabaseLogin>,
}

impl Database {
    pub async fn create() -> Result<Self> {
        let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
        let client = Client::with_options(client_options)?;
        let db = client.database("rschat");
        let messages = db.collection("messages");
        let channels = db.collection("channels");
        let users = db.collection("users");
        let logins = db.collection("logins");
        Ok(Self {
            client,
            db,
            messages,
            channels,
            users,
            logins,
        })
    }
}

pub async fn to_vec<T>(
    mut cursor: Cursor<T>,
) -> std::result::Result<Vec<T>, <mongodb::Cursor<T> as TryStream>::Error>
where
    Cursor<T>: TryStreamExt,
    Cursor<T>: TryStream,
    T: std::marker::Unpin,
    <mongodb::Cursor<T> as TryStream>::Ok: Into<T>,
{
    let mut out: Vec<T> = vec![];

    while let Some(val) = cursor.try_next().await? {
        out.push(val.into());
    }

    Ok(out)
}
