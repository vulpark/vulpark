use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{WebSocket, Ws},
    Rejection, Reply,
};

use crate::structures::Event;

use super::{Client, Clients, with_lock};

#[derive(Debug, Deserialize)]
enum ReceivedEvent {
    Handshake {
        username: String
    },
}

impl ReceivedEvent {
    async fn handle(&self, client_id: String, clients: Clients) -> Option<Event> {
        match self {
            Self::Handshake { username } => {
                let mut lock = with_lock!(clients);
                let client = lock.get_mut(&client_id).unwrap();
                if let Some(_) = client.username {
                    return None;
                }
                client.username = Some(username.clone());
                Some(Event::HandshakeComplete { username: username.clone() })
            }
        }
    }
}

pub async fn gateway(ws: Ws, clients: Clients) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |socket| {
        let client = Client::empty();
        let id = client.id.clone();
        handle_conn(socket, clients, client, id)
    }))
}

async fn handle_conn(ws: WebSocket, clients: Clients, mut client: Client, id: String) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    with_lock!(clients).insert(id.clone(), client);

    println!("{} connected", id);

    let _ = &with_lock!(clients)
        .get(&id)
        .unwrap()
        .send(&Event::start_handshake(id.clone()));

    macro disconnect() {
        let name = with_lock!(clients).remove(&id).unwrap().get_name();
        println!("disconnecting {}", name);
    }

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(err) => {
                println!("{}", err);
                break;
            }
        };

        if msg.is_text() && let Ok(event) = serde_json::from_str::<ReceivedEvent>(msg.to_str().unwrap()) {
            println!("{:?}", event);
            let event = event.handle(id.clone(), clients.clone()).await;
            if let None = event {
                continue;
            }
            let _ = &with_lock!(clients)
                .get(&id)
                .unwrap()
                .send(&event.unwrap());
        }
    }

    disconnect!();
}
