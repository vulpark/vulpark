// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;

use crate::generate_ulid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub id: String,
    pub service: Service,
    pub service_user: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Service {
    #[serde(rename = "github")]
    Github,
}

#[derive(Debug)]
pub enum AuthError {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    String(String),
}

impl Login {
    pub fn new(service: Service, service_user: String, user_id: String) -> Self {
        Self {
            id: generate_ulid(),
            service,
            service_user,
            user_id,
        }
    }
}

impl Service {
    pub fn secret(self) -> String {
        std::env::var(format!("{self}_SECRET").to_uppercase()).unwrap()
    }

    pub fn client_id(self) -> String {
        match self {
            Self::Github => "01a54a05ac326eca7c4f".to_string(),
        }
    }

    pub fn token_url(self, code: &str) -> String {
        match self {
            Self::Github => format!(
                "https://github.com/login/oauth/access_token?client_id={}&code={}&client_secret={}",
                self.client_id(),
                code,
                self.secret()
            ),
        }
    }

    pub fn user_url(self) -> String {
        match self {
            Self::Github => "https://api.github.com/user".to_string(),
        }
    }

    pub async fn fetch_token(self, code: &str) -> Result<String, AuthError> {
        let url = self.token_url(code);
        match self {
            Self::Github => self.fetch_token_gh(url).await,
        }
    }

    async fn fetch_token_gh(self, url: String) -> Result<String, AuthError> {
        let client = reqwest::Client::new();
        let body = client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await?
            .text()
            .await?;
        let response: Value = serde_json::from_str(&body)?;
        Ok(response
            .get("access_token")
            .ok_or(AuthError::String("No token field.".to_string()))?
            .as_str()
            .ok_or(AuthError::String("Token field not string".to_string()))?
            .to_string())
    }

    pub async fn get_uid(self, token: String) -> Result<String, AuthError> {
        let url = self.user_url();
        match self {
            Self::Github => self.get_uid_gh(url, token).await,
        }
    }

    async fn get_uid_gh(self, url: String, token: String) -> Result<String, AuthError> {
        let client = reqwest::Client::new();
        let f = format!("Bearer {token}");
        println!("{f}");
        let body = client
            .get(url)
            .header("Authorization", f)
            .header("User-Agent", "Vulpark Authentication Services")
            .send()
            .await?
            .text()
            .await?;
        let response: Value = serde_json::from_str(&body)?;
        Ok(response
            .get("id")
            .ok_or(AuthError::String("No user ID field.".to_string()))?
            .as_u64()
            .ok_or(AuthError::String("User ID field wrong type.".to_string()))?
            .to_string())
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Github => f.write_str("github"),
        }
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl ToString for AuthError {
    fn to_string(&self) -> String {
        match self {
            Self::Reqwest(err) => err.to_string(),
            Self::Serde(err) => err.to_string(),
            Self::String(str) => str.to_string(),
        }
    }
}
