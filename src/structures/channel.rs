// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::error::Error;
use rweb::Schema;
use serde::{
    de::{self, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

use crate::{database, database::DatabaseGuildResponse, generate_ulid};

use super::restricted_string::RestrictedString;

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub location: ChannelLocation,
}

#[derive(Debug, Clone, Schema)]
pub enum ChannelLocation {
    Dm { members: Vec<String> },
    Guild { guild: String },
}

#[derive(Debug, Deserialize, Schema)]
pub struct ChannelCreate {
    pub name: String,
    pub location: ChannelLocation,
}

#[derive(Debug, Clone, Serialize, Schema)]
pub struct ChannelResponse {
    pub channel: Channel,
}

impl Channel {
    pub fn new(name: &str, location: ChannelLocation) -> Self {
        Self {
            id: generate_ulid(),
            name: RestrictedString::no_space(name),
            location,
        }
    }

    pub async fn insert(self) -> Result<Self, Error> {
        database().await.create_channel(self).await
    }

    pub async fn get_users(&self) -> DatabaseGuildResponse<Vec<String>> {
        DatabaseGuildResponse::Ok(match &self.location {
            ChannelLocation::Dm { members } => members.clone(),
            ChannelLocation::Guild { guild } => database()
                .await
                .fetch_guild_users(guild)
                .await
                .unwrap_or(DatabaseGuildResponse::NoGuild)?
                .iter()
                .map(|it| it.id.clone())
                .collect(),
        })
    }
}

impl Serialize for ChannelLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Dm { members } => {
                let mut ser = serializer.serialize_struct("ChannelLocation", 2)?;
                ser.serialize_field("type", "dm")?;
                ser.serialize_field("members", members)?;
                ser.end()
            }
            Self::Guild { guild } => {
                let mut ser = serializer.serialize_struct("ChannelLocation", 2)?;
                ser.serialize_field("type", "guild")?;
                ser.serialize_field("id", guild)?;
                ser.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ChannelLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(LocationVisitor)
    }
}

struct LocationVisitor;

impl<'de> Visitor<'de> for LocationVisitor {
    type Value = ChannelLocation;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a channel location.")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut name: Option<&str> = None;
        let mut members: Vec<String> = vec![];
        let mut id: Option<String> = None;
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "type" => name = Some(map.next_value()?),
                "members" => members = map.next_value()?,
                "id" => id = Some(map.next_value()?),
                _ => {}
            }
        }
        let Some(name) = name else {
            return Err(de::Error::missing_field("type"))
        };
        match name {
            "dm" => Ok(ChannelLocation::Dm { members }),
            "guild" => {
                if let Some(id) = id {
                    Ok(ChannelLocation::Guild { guild: id })
                } else {
                    Err(de::Error::missing_field("id"))
                }
            }
            _ => Err(de::Error::unknown_variant("type", &["dm", "guild"])),
        }
    }
}

impl ChannelResponse {
    pub fn from_channel(channel: Channel) -> Self {
        Self { channel }
    }
}
