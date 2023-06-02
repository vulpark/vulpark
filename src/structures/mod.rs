pub mod message;

use serde::Serialize;

use self::message::Message;

#[derive(Serialize)]
pub enum Event {
    HandshakeStart { client_id: String },
    HandshakeComplete { username: String },
    MessageCreate { message: Message },
}

impl Event {
    pub fn start_handshake(client_id: String) -> Self {
        Self::HandshakeStart { client_id }
    }
}

impl ToString for Event {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
