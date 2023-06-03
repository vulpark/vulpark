pub mod channel;
pub mod message;
pub mod restricted_string;
pub mod user;

use serde::Serialize;

use self::{message::Message, user::User};

#[derive(Serialize)]
pub enum Event {
    HandshakeStart {},
    HandshakeComplete {
        user: User,
    },
    MessageCreate {
        message: Message,
        author: Option<User>,
    },
}

impl ToString for Event {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
