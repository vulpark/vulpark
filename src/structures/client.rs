// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::{mpsc, Mutex};
use warp::ws::Message;

use crate::{database, generate_ulid, with_lock};

use super::{event::Event, user::User};

#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
    pub user_id: Option<String>,
}

impl Client {
    pub fn empty() -> Self {
        Client {
            id: generate_ulid(),
            sender: None,
            user_id: None,
        }
    }

    pub fn send(&self, event: &Event) {
        let Some(ref sender) = self.sender else { return };
        let _ = sender.send(Ok(Message::text(event.to_string())));
    }

    pub async fn set_user(&mut self, token: String) -> Option<User> {
        let user = database().await.fetch_user_token(token.clone()).await;
        let Ok(user) = user else {
            return None;
        };
        let user = user?;

        self.user_id = Some(user.id.clone());
        Some(user)
    }

    pub async fn remove_from(&self, clients: ClientHolder) -> Option<()> {
        let mut lock = with_lock!(clients);
        let id = self.user_id.clone()?;
        let clients = lock.get_mut(&id)?;
        let index = clients.into_iter().position(|it| it.id == self.id)?;
        clients.remove(index);
        Some(())
    }
}

pub struct Clients(pub HashMap<String, Vec<Client>>);

impl Clients {
    pub fn dispatch_global(&self, event: Event) {
        self.values()
            .for_each(|clients| Self::dispatch_to(clients, &event));
    }

    pub fn dispatch_users(&self, users: Vec<String>, event: Event) {
        users.into_iter().for_each(|user| {
            if let Some(clients) = self.get(&user) {
                Self::dispatch_to(clients, &event)
            }
        })
    }

    fn dispatch_to(clients: &Vec<Client>, event: &Event) {
        clients.into_iter().for_each(|client| client.send(event))
    }
}

impl Deref for Clients {
    type Target = HashMap<String, Vec<Client>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Clients {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type ClientHolder = Arc<Mutex<Clients>>;
