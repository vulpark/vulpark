mod message;

use mongodb::{bson::doc, error::Result, options::ClientOptions, Client, Collection};

use crate::structures::message::Message;

use self::message::DatabaseMessage;

#[allow(dead_code)]
pub struct Database {
    client: Client,
    db: mongodb::Database,
    messages: Collection<DatabaseMessage>,
}

impl Database {
    pub async fn create() -> Result<Self> {
        let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
        let client = Client::with_options(client_options)?;
        let db = client.database("rschat");
        let messages = db.collection("messages");
        Ok(Self {
            client,
            db,
            messages,
        })
    }

    pub async fn create_message(&self, message: Message) -> Result<Message> {
        self.messages
            .insert_one(DatabaseMessage::from(&message), None)
            .await?;
        Ok(message)
    }

    pub async fn fetch_message(&self, id: String) -> Result<Option<Message>> {
        let Some(message) = self.messages.find_one(doc!{"_id": id}, None).await? else {
            return Ok(None)
        };
        Ok(Some(message.into()))
    }
}
