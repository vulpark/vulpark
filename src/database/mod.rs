mod macros;
mod message;
mod user;

use mongodb::bson::doc;
use mongodb::{error::Result, options::ClientOptions, Client, Collection};
use rand::Rng;
use ulid::Ulid;

use crate::structures::message::Message;
use crate::structures::user::User;

use self::macros::*;
use self::message::DatabaseMessage;
use self::user::DatabaseUser;

#[allow(dead_code)]
pub struct Database {
    client: Client,
    db: mongodb::Database,
    messages: Collection<DatabaseMessage>,
    users: Collection<DatabaseUser>,
}

macro basic_fetch($col: expr, $id: expr) {
    if let Some(val) = $col.find_one($id, None).await? {
        Some(val.into())
    } else {
        None
    }
}

impl Database {
    pub async fn create() -> Result<Self> {
        let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
        let client = Client::with_options(client_options)?;
        let db = client.database("rschat");
        let messages = db.collection("messages");
        let users = db.collection("users");
        Ok(Self {
            client,
            db,
            messages,
            users,
        })
    }

    pub async fn create_message(&self, message: Message) -> Result<Message> {
        self.messages
            .insert_one(DatabaseMessage::from(&message), None)
            .await?;
        Ok(message)
    }

    pub async fn fetch_message(&self, id: String) -> Result<Option<Message>> {
        Ok(basic_fetch!(self.messages, id!(id)))
    }

    async fn create_user_internal(&self, username: String) -> Result<Option<DatabaseUser>> {
        let mut discrim: u32 = rand::thread_rng().gen_range(1..9999);
        let mut count = 1;
        while let Some(_) = self
            .users
            .find_one(
                doc! {"username": username.clone(), "discriminator": discrim},
                None,
            )
            .await?
        {
            discrim = rand::thread_rng().gen_range(1..9999);
            count += 1;
            if count == 9999 {
                return Ok(None);
            }
        }
        let user = DatabaseUser {
            _id: Ulid::new().to_string(),
            username,
            discriminator: discrim,
            token: Ulid::new().to_string(),
        };
        self.users.insert_one(user.clone(), None).await?;
        Ok(Some(user))
    }

    pub async fn create_user(&self, username: String) -> Result<Option<(User, String)>> {
        let Some(user) = self.create_user_internal(username).await? else {
            return Ok(None)
        };
        Ok(Some((user.clone().into(), user.token)))
    }

    pub async fn fetch_user(&self, id: String) -> Result<Option<User>> {
        Ok(basic_fetch!(self.users, id!(id)))
    }

    pub async fn fetch_user_token(&self, token: String) -> Result<Option<User>> {
        Ok(basic_fetch!(self.users, eq!("token", token)))
    }
}
