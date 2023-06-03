use serde::{Deserialize, Serialize};

use super::restricted_string::RestrictedString;

#[derive(Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: RestrictedString,
}
