pub mod message;

use serde::Serialize;

use self::message::Message;

#[derive(Serialize)]
pub enum Event {
    LoginRequest { client_id: String },
    LoginSuccess { username: String },
    MessageCreate { message: Message },
}

impl Event {
    pub fn start_login(client_id: String) -> Self {
        Self::LoginRequest { client_id }
    }
}

impl ToString for Event {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
