// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use futures::StreamExt;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{WebSocket, Ws},
    Rejection, Reply,
};

use crate::structures::Event;

use super::{with_lock, Client, ClientHolder};

#[derive(Debug, Deserialize)]
enum ReceivedEvent {
    Handshake { token: String },
}

impl ReceivedEvent {
    async fn handle(&self, mut client: Client, clients: &ClientHolder) -> Option<Event> {
        match self {
            Self::Handshake { token } => {
                if let Some(_) = client.user_id {
                    return None;
                }
                let user = client.set_user(token.clone()).await?;
                {
                    let mut lock = with_lock!(clients);
                    if let Some(clients) = lock.get_mut(&user.id) {
                        clients.push(client.clone());
                    } else {
                        lock.insert(user.id.clone(), vec![client.clone()]);
                    };
                }
                Some(Event::HandshakeComplete { user })
            }
        }
    }
}

pub async fn gateway(ws: Ws, clients: ClientHolder) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |socket| {
        let client = Client::empty();
        handle_conn(socket, clients, client)
    }))
}

async fn handle_conn(ws: WebSocket, clients: ClientHolder, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender));

    client.sender = Some(client_sender);

    client.send(&Event::HandshakeStart {});

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(_) => break,
        };

        if msg.is_text() && let Ok(event) = serde_json::from_str::<ReceivedEvent>(msg.to_str().unwrap()) {
            let event = event.handle(client.clone(), &clients).await;
            if let None = event {
                continue;
            }
            let _ = client.send(&event.unwrap());
        }
    }

    let _ = client.remove_from(clients);
}
