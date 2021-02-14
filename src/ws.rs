extern crate redis;
use crate::{Client, Clients};
use futures::{FutureExt, StreamExt};
use serde::Deserialize;
// use serde_json::from_str;
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};
use redis::{Client as RClient, ControlFlow, PubSubCommands};
use std::sync::Arc;
use std::thread;
// use std::time::Duration;

#[derive(Deserialize, Debug)]
pub struct TopicsRequest {
    topics: Vec<String>,
}

trait AppState {
    fn client(&self) -> &Arc<RClient>;
}

struct Ctx {
    pub client: Arc<RClient>,
}

impl Ctx {
    fn new() -> Ctx {
        let client = RClient::open("redis://localhost:6379").unwrap();
        Ctx {
            client: Arc::new(client),
        }
    }
}

impl AppState for Ctx {
    fn client(&self) -> &Arc<RClient> {
        &self.client
    }
}
//
// fn subscribe(state: &impl AppState) -> thread::JoinHandle<()> {
//     let client = Arc::clone(state.client());
//     thread::spawn(move || {
//         let mut conn = client.get_connection().unwrap();
//
//         conn.subscribe(&["boo"], |msg| {
//             let ch = msg.get_channel_name();
//             let payload: String = msg.get_payload().unwrap();
//             match payload.as_ref() {
//                 "10" => ControlFlow::Break(()),
//                 a => {
//                     println!("Channel '{}' received '{}'.", ch, a);
//                     ControlFlow::Continue
//                 }
//             }
//         }).unwrap();
//     })
// }



pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, client: Client) {
    let (client_ws_sender, mut _client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    // client.sender = Some(client_sender);
    clients.write().await.insert(id.clone(), client);
    println!("{} connected", id);
    let ctx = Ctx::new();
    let rclient = Arc::clone(ctx.client());
    let mut conn = rclient.get_connection().unwrap();
    thread::spawn(move || {
        conn.subscribe(&["boo"], |msg| {
            let ch = msg.get_channel_name();
            let payload: String = msg.get_payload().unwrap();
            let message_text = format!("Channel '{}' received '{}'.", ch, payload);

            let _ = client_sender.send(Ok(Message::text(message_text.clone())));
            println!("Channel '{}' received '{}'", ch, payload);

            match payload.as_ref() {
                "10" => ControlFlow::Break(()),
                _a => {
                    ControlFlow::Continue
                }
            }
        }).unwrap();
    });
    // println!("{} disconnected", id);
}
