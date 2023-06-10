// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mongodb::bson::doc;
use mongodb::error::Result;
use serde::{Deserialize, Serialize};

use crate::structures::auth::{Login, Service};

use super::{macros::{basic_create, basic_fetch, eq}, to_vec, Database};

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseLogin {
    pub _id: String,
    pub service: Service,
    pub service_user: String,
    pub user_id: String,
}

impl From<&Login> for DatabaseLogin {
    fn from(value: &Login) -> Self {
        Self {
            _id: value.id.to_string(),
            service: value.service,
            service_user: value.service_user.clone(),
            user_id: value.user_id.clone(),
        }
    }
}

impl From<DatabaseLogin> for Login {
    fn from(value: DatabaseLogin) -> Login {
        Login {
            id: value._id,
            service: value.service,
            service_user: value.service_user,
            user_id: value.user_id,
        }
    }
}

impl Database {
    pub async fn create_login(&self, login: Login) -> Result<Login> {
        basic_create!(self.logins, DatabaseLogin::from, login)
    }

    pub async fn fetch_logins(&self, user_id: String, service: Service) -> Result<Vec<Login>> {
        let service = service.to_string();
        let Ok(vec) = to_vec(self.logins.find(eq!(user_id, service), None).await?).await else {
            return Ok(vec![])
        };
        Ok(vec.into_iter().map(Into::into).collect())
    }

    pub async fn fetch_login(&self, service: Service, service_user: String) -> Result<Option<Login>> {
        let service = service.to_string();
        basic_fetch!(self.logins, eq!(service, service_user))
    }
}
