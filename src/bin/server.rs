use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};
use serde::{Deserialize, Serialize};

// Struct untuk parse pesan dari client
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: String,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

// Shared state: list username yang connected
type UserList = Arc<Mutex<Vec<String>>>;

async fn broadcast_user_list(
    users: &UserList,
    bcast_tx: &Sender<String>,
) {
    let user_list = users.lock().unwrap().clone();
    let msg = WebSocketMessage {
        message_type: "users".to_string(),
        data_array: Some(user_list),
        data: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let _ = bcast_tx.send(json);
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    users: UserList,
) {
    let mut username: Option<String> = None;
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From client {addr:?}: {text}");

                            // Parse JSON dari client
                            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                                match ws_msg.message_type.as_str() {

                                    // Client register dengan username
                                    "register" => {
                                        if let Some(name) = ws_msg.data {
                                            println!("User registered: {name}");
                                            username = Some(name.clone());

                                            // Tambah ke user list
                                            users.lock().unwrap().push(name);

                                            // Broadcast user list terbaru ke semua
                                            broadcast_user_list(&users, &bcast_tx).await;
                                        }
                                    }

                                    // Client kirim pesan chat
                                    "message" => {
                                        if let Some(text_msg) = ws_msg.data {
                                            let from = username
                                                .clone()
                                                .unwrap_or_else(|| addr.to_string());

                                            // Bungkus pesan dengan format JSON
                                            let reply = serde_json::json!({
                                                "messageType": "message",
                                                "dataArray": null,
                                                "data": serde_json::json!({
                                                    "from": from,
                                                    "message": text_msg
                                                }).to_string()
                                            });

                                            let _ = bcast_tx.send(reply.to_string());
                                        }
                                    }

                                    _ => {
                                        println!("Unknown message type: {}", ws_msg.message_type);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        println!("Client {addr:?} disconnected");

                        // Hapus dari user list saat disconnect
                        if let Some(name) = &username {
                            users.lock().unwrap().retain(|u| u != name);
                            broadcast_user_list(&users, &bcast_tx).await;
                        }
                        break;
                    }
                }
            }

            // Broadcast pesan ke client ini
            msg = bcast_rx.recv() => {
                if let Ok(text) = msg {
                    ws_stream
                        .send(Message::text(text))
                        .await
                        .unwrap_or_default();
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);
    let users: UserList = Arc::new(Mutex::new(Vec::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");

        let bcast_tx = bcast_tx.clone();
        let users = users.clone();

        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await.unwrap();
            handle_connection(addr, ws_stream, bcast_tx, users).await;
        });
    }
}