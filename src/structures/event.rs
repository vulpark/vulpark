// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::Deserialize;

use super::{channel::ChannelResponse, guild::GuildResponse, message::MessageResponse, user::User};

macro_rules! event {
    ($($name:ident $({ $($n: ident: $t_1:ty),* $(,)? })? $(($t_2:ty))? ),+ $(,)?) => {
        #[derive(rweb::Schema)]
        pub enum Event {
            $(
                $name $({$($n: $t_1)*})? $(($t_2))?
            ),+
        }

        impl serde::Serialize for Event {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer {
                use serde::ser::{SerializeStruct, SerializeStructVariant};
                use std::collections::HashMap;
                $(
                    event!(ser|| self, serializer, $name $(, $($n),*)? $(, $t_2)?);
                )+
                panic!();
            }
        }
    };

    (ser|| $self:expr, $serializer:expr, $name:ident) => {
        if let Self::$name = $self {
            let mut ser = $serializer.serialize_struct("Event", 1)?;
            ser.serialize_field(stringify!($name), &HashMap::<String, ()>::new())?;
            return ser.end()
        }
    };

    (ser|| $self:expr, $serializer:expr, $name:ident, $($n:ident),*) => {
        if let Self::$name { $($n),* } = $self {
            let mut ser = $serializer.serialize_struct_variant("Event", 0, stringify!($name), event!(size|| $($n),*))?;
            $(
                ser.serialize_field(stringify!($n), $n)?;
            )*
            return ser.end()
        }
    };

    (ser|| $self:expr, $serializer:expr, $name:ident, $t:ty) => {
        if let Self::$name(val) = $self {
            let mut ser = $serializer.serialize_struct("Event", 1)?;
            ser.serialize_field(stringify!($name), val)?;
            return ser.end()
        }
    };

    (size||) => {
        0usize
    };

    (size|| $n: ident $(, $n_:ident),*) => {
        1usize + event!(size|| $($n_: ident),*)
    };
}

event! {
    HandshakeStart,
    HandshakeComplete {
        user: User,
    },
    MessageCreate (MessageResponse),
    ChannelCreate (ChannelResponse),
    GuildCreate (GuildResponse),
}

#[derive(Debug, Deserialize)]
pub enum ReceivedEvent {
    Handshake { token: String },
}

impl ToString for Event {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
