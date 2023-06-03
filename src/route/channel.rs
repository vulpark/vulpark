use serde::Deserialize;

use crate::structures::restricted_string::RestrictedString;

use super::Clients;

#[derive(Debug, Deserialize)]
pub struct ChannelCreate {
    name: RestrictedString,
}

pub fn create(token: String, create: ChannelCreate, clients: Clients) {}

pub fn fetch(token: String, id: String) {}
