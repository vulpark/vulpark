use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{WebSocket, Ws},
    Rejection, Reply,
};

use crate::structures::Event;

use super::{Client, Clients};

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
    clients.lock().await.insert(id.clone(), client);

    println!("{} connected", id);

    let _ = &clients
        .lock()
        .await
        .get(&id)
        .unwrap()
        .send(&Event::start_login(id.clone()));

    while let Some(result) = client_ws_rcv.next().await {
        match result {
            Ok(msg) => msg,
            Err(e) => {
                let name = clients.lock().await.get(&id).unwrap().get_name();
                eprintln!("Error receiving ws message for {}: {}", name, e);
                break;
            }
        };
    }

    let name = clients.lock().await.remove(&id).unwrap().get_name();
    println!("{} disconnected", name);
}
