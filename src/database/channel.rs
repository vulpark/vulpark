use serde::{Deserialize, Serialize};

use crate::structures::{channel::Channel, restricted_string::RestrictedString};

#[derive(Serialize, Deserialize)]
pub struct DatabaseChannel {
    pub _id: String,
    pub name: RestrictedString,
}

impl From<&Channel> for DatabaseChannel {
    fn from(value: &Channel) -> Self {
        Self {
            _id: value.id.to_string(),
            name: value.name.clone(),
        }
    }
}

impl Into<Channel> for DatabaseChannel {
    fn into(self) -> Channel {
        Channel {
            id: self._id,
            name: self.name,
        }
    }
}
