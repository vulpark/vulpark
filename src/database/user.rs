use serde::{Deserialize, Serialize};

use crate::structures::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub(super) _id: String,
    pub(super) username: String,
    pub(super) discriminator: u32,
    pub(super) token: String,
}

impl Into<User> for DatabaseUser {
    fn into(self) -> User {
        User {
            id: self._id,
            username: self.username,
            discriminator: self.discriminator.to_string(),
        }
    }
}
