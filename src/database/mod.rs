// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::used_underscore_binding)]

mod auth;
mod channel;
mod guild;
mod macros;
mod message;
mod user;

pub use guild::DatabaseGuildResponse;

use futures::stream::TryStreamExt;
use futures::TryStream;
use mongodb::Cursor;
use mongodb::{error::Result, options::ClientOptions, Client, Collection};

use self::auth::DatabaseLogin;
use self::channel::DatabaseChannel;
use self::guild::DatabaseGuild;
use self::message::DatabaseMessage;
use self::user::DatabaseUser;

/// This is using old syntax because it doesn't work with new syntax.
macro_rules! db {
    { $( $i: ident : $t: ty ),* $(,)? } => {
        pub struct Database {
            $(
                pub $i : Collection<$t>,
            )*
        }

        impl Database {
            pub async fn create() -> Result<Self> {
                let client_options = ClientOptions::parse(std::env::var("DB_URL").expect("No DB_URL found in environment!")).await?;
                let client = Client::with_options(client_options)?;
                let db = client.default_database().expect("No database specified in connection string");
                Ok(Self {
                    $(
                        $i: db.collection(stringify!($i)),
                    )*
                })
            }
        }
    };
}

db! {
    messages: DatabaseMessage,
    channels: DatabaseChannel,
    users: DatabaseUser,
    logins: DatabaseLogin,
    guilds: DatabaseGuild,
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
