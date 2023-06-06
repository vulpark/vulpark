// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::error::Error;
use serde::{Deserialize, Serialize, ser::SerializeStruct, de::{Visitor, self}};

use crate::{database, generate_ulid};

use super::{restricted_string::RestrictedString};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: RestrictedString,
    pub location: ChannelLocation
}

impl Channel {
    pub fn new(name: RestrictedString, location: ChannelLocation) -> Self {
        Self {
            id: generate_ulid(),
            name,
            location,
        }
    }

    pub async fn insert(self) -> Result<Self, Error> {
        database().await.create_channel(self).await
    }
}

#[derive(Debug, Clone)]
pub enum ChannelLocation {
    Dm {
        members: Vec<String>
    },
}

impl Serialize for ChannelLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match self {
            Self::Dm { members } => {
                let mut ser = serializer.serialize_struct("ChannelLocation", 2)?;
                ser.serialize_field("type", "dm")?;
                ser.serialize_field("members", members)?;
                ser.end()
            }
        }

    }
}

impl <'de> Deserialize<'de> for ChannelLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_map(LocationVisitor)
    }
}

struct LocationVisitor;

impl <'de> Visitor<'de> for LocationVisitor {
    type Value = ChannelLocation;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a channel location.")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        let mut name: Option<&str> = None;
        let mut members: Vec<String> = vec![];
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "type" => name = Some(map.next_value()?),
                "members" => members = map.next_value()?,
                _ => {},
            }
        }
        let Some(name) = name else {
            return Err(de::Error::missing_field("type"))
        };
        match name {
            "dm" => Ok(ChannelLocation::Dm {
                members
            }),
            _ => Err(de::Error::unknown_variant("type", &["dm"]))
        }
    }
}
