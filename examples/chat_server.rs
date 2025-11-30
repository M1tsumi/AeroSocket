//! Chat Server Example
//!
//! This example demonstrates a simple multi-client chat server that broadcasts
//! messages to all connected clients.

use aerosocket::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

type ClientMap = Arc<RwLock<HashMap<u64, ConnectionHandle>>>;

#[derive(Clone)]
struct ChatServer {
    clients: ClientMap,
    message_sender: broadcastSender<String>,
}

impl ChatServer {
    fn new() -> Self {
        let (message_sender, _) = broadcast::channel(1000);
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            message_sender,
        }
    }

    async fn add_client(&self, id: u64, conn: ConnectionHandle) {
        self.clients.write().await.insert(id, conn);
        self.broadcast(&format!("User {} joined the chat", id)).await;
    }

    async fn remove_client(&self, id: u64) {
        self.clients.write().await.remove(&id);
        self.broadcast(&format!("User {} left the chat", id)).await;
    }

    async fn broadcast(&self, message: &str) {
        let _ = self.message_sender.send(message.to_string());
    }

    async fn handle_client(&self, id: u64, mut conn: Connection) -> Result<()> {
        let mut receiver = self.message_sender.subscribe();
        
        // Handle incoming messages
        let handle = tokio::spawn(async move {
            while let Some(msg) = conn.next().await? {
                match msg {
                    Message::Text(text) => {
                        let broadcast_msg = format!("User {}: {}", id, text);
                        self.broadcast(&broadcast_msg).await;
                    }
                    Message::Close(_, _) => break,
                    _ => {}
                }
            }
            Ok::<(), Error>(())
        });

        // Handle outgoing messages
        let receiver_handle = tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                if let Err(_) = conn.send_text(&msg).await {
                    break;
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            result = handle => {
                if let Err(e) = result {
                    eprintln!("Error handling client {}: {:?}", id, e);
                }
            }
            _ = receiver_handle => {}
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let chat_server = ChatServer::new();
    let mut client_counter = 0;

    let server = Server::builder()
        .bind("127.0.0.1:8080")
        .max_connections(100)
        .build()?;

    println!("💬 Chat server listening on ws://127.0.0.1:8080");

    server.serve(|conn| async move {
        let client_id = client_counter;
        client_counter += 1;

        println!("👤 Client {} connected from {}", client_id, conn.remote_addr());
        
        if let Err(e) = chat_server.handle_client(client_id, conn).await {
            eprintln!("Error handling client {}: {:?}", client_id, e);
        }
        
        chat_server.remove_client(client_id).await;
        println!("👤 Client {} disconnected", client_id);
        
        Ok(())
    }).await?;

    Ok(())
}
