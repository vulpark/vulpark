// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use futures::StreamExt;
use rweb::*;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{WebSocket, Ws},
    Filter, Rejection, Reply,
};

use crate::{
    database,
    structures::{
        client::{Client, ClientHolder},
        event::Event,
        event::ReceivedEvent,
    },
    with_lock,
};

pub fn routes(
    clients: &ClientHolder,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    gateway(clients.clone())
}

#[get("/gateway")]
pub async fn gateway(
    #[filter = "ws"] ws: Ws,
    #[data] clients: ClientHolder
) -> Result<impl Reply, Rejection> {
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

    client.send(&Event::HandshakeStart);

    while let Some(result) = client_ws_rcv.next().await {
        let Ok(msg) = result else {
            break;
        };

        if msg.is_text() && let Ok(event) = serde_json::from_str::<ReceivedEvent>(msg.to_str().unwrap()) {
            let event = handle_event(&event, client.clone(), &clients).await;
            if event.is_none() {
                continue;
            }
            client.send(&event.unwrap());
        }
    }

    client.remove_from(clients).await;
}

async fn handle_event(
    event: &ReceivedEvent,
    mut client: Client,
    clients: &ClientHolder,
) -> Option<Event> {
    match event {
        ReceivedEvent::Handshake { token } => {
            if client.user_id.is_some() {
                return None;
            }
            let user = client.set_user(token.clone()).await?;
            let _ = database()
                .await
                .set_user_gateway_connected(&user.id, true)
                .await;
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
