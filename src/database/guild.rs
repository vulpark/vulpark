// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ops::{ControlFlow, FromResidual, Try};

use mongodb::error::Result;
use serde::{Deserialize, Serialize};

use crate::{
    database::macros::{eq_keyed, id},
    structures::{guild::Guild, user::User},
};

use super::{
    macros::{basic_create, basic_fetch, basic_update, keyed},
    to_vec,
    user::DatabaseUser,
    Database,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseGuild {
    pub _id: String,
    pub name: String,
    pub owner_id: String,
}

#[derive(Debug, Clone)]
pub enum DatabaseGuildResponse<T> {
    NoUser,
    NoGuild,
    Ok(T),
}

impl From<&Guild> for DatabaseGuild {
    fn from(value: &Guild) -> Self {
        Self {
            _id: value.id.clone(),
            name: value.name.clone(),
            owner_id: value.owner_id.clone(),
        }
    }
}

impl From<DatabaseGuild> for Guild {
    fn from(value: DatabaseGuild) -> Self {
        Self {
            id: value._id,
            name: value.name,
            owner_id: value.owner_id,
        }
    }
}

impl Database {
    pub async fn create_guild(&self, guild: Guild) -> Result<Guild> {
        basic_create!(self.guilds, DatabaseGuild::from, guild)
    }

    pub async fn fetch_guild(&self, id: &str) -> Result<Option<Guild>> {
        basic_fetch!(self.guilds, id!(id))
    }

    pub async fn fetch_guild_users(&self, id: &str) -> Result<DatabaseGuildResponse<Vec<User>>> {
        if self.fetch_guild(id).await?.is_none() {
            return Ok(DatabaseGuildResponse::NoGuild);
        }
        let v: Vec<DatabaseUser> =
            to_vec(self.users.find(keyed!("guilds", id), None).await?).await?;

        Ok(DatabaseGuildResponse::Ok(
            v.iter().map(User::from).collect(),
        ))
    }

    pub async fn fetch_guild_connected_users(
        &self,
        id: &str,
    ) -> Result<DatabaseGuildResponse<Vec<User>>> {
        if self.fetch_guild(id).await?.is_none() {
            return Ok(DatabaseGuildResponse::NoGuild);
        }
        let v: Vec<DatabaseUser> = to_vec(
            self.users
                .find(keyed!("guilds", id, "gateway_connected", true), None)
                .await?,
        )
        .await?;

        Ok(DatabaseGuildResponse::Ok(
            v.iter().map(User::from).collect(),
        ))
    }

    pub async fn fetch_guilds_from_user(
        &self,
        user: &str,
    ) -> Result<DatabaseGuildResponse<Vec<Guild>>> {
        let Some(user): Option<DatabaseUser> = basic_fetch!(self.users, id!(user))? else {
            return Ok(DatabaseGuildResponse::NoUser)
        };

        let mut guilds = vec![];
        let mut user_guilds = vec![];

        for id in &user.guilds {
            if let Some(guild) = self.fetch_guild(id).await? {
                guilds.push(guild);
                user_guilds.push(id);
            }
        }

        if guilds.len() != user.guilds.len() {
            let _: Option<User> =
                basic_update!(self.users, id!(user._id), eq_keyed!("guilds", user_guilds))?;
        }

        Ok(DatabaseGuildResponse::Ok(guilds))
    }

    pub async fn join_guild(&self, id: &str, user: &str) -> Result<DatabaseGuildResponse<User>> {
        if self.fetch_guild(id).await?.is_none() {
            return Ok(DatabaseGuildResponse::NoGuild);
        }
        let Some(mut user): Option<DatabaseUser> = basic_fetch!(self.users, id!(user))? else {
            return Ok(DatabaseGuildResponse::NoUser)
        };

        let id = id.to_string();
        if !user.guilds.contains(&id) {
            user.guilds.push(id);
            self.users
                .find_one_and_update(id!(&user._id), keyed!("guilds", &user.guilds), None)
                .await?;
        }

        Ok(DatabaseGuildResponse::Ok(user.into()))
    }
}

impl<T> DatabaseGuildResponse<T> {
    pub fn option(self) -> Option<T> {
        if let Self::Ok(t) = self {
            return Some(t);
        }
        None
    }

    pub fn unwrap_or(self, default: T) -> T {
        if let Self::Ok(t) = self {
            return t;
        }
        default
    }
}

impl<T> Try for DatabaseGuildResponse<T> {
    type Output = T;

    type Residual = DatabaseGuildResponse<std::convert::Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(output) => ControlFlow::Continue(output),
            Self::NoGuild => ControlFlow::Break(DatabaseGuildResponse::NoGuild),
            Self::NoUser => ControlFlow::Break(DatabaseGuildResponse::NoUser),
        }
    }
}

impl<T> FromResidual for DatabaseGuildResponse<T> {
    fn from_residual(residual: <Self as std::ops::Try>::Residual) -> Self {
        match residual {
            DatabaseGuildResponse::NoGuild => Self::NoGuild,
            DatabaseGuildResponse::NoUser => Self::NoUser,
            DatabaseGuildResponse::Ok(_) => panic!("Infallible for DatabaseGuildResponse met!"),
        }
    }
}
