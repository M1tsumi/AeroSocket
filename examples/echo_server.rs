//! Echo Server Example
//!
//! This example demonstrates a simple WebSocket server that echoes back
//! any messages it receives from clients.

use aerosocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create the server
    let server = Server::builder()
        .bind("127.0.0.1:8080")
        .max_connections(1000)
        .compression(true)
        .build()?;

    println!("🚀 Echo server listening on ws://127.0.0.1:8080");

    // Start serving connections
    server.serve(|mut conn| async move {
        println!("📡 New connection from {}", conn.remote_addr());

        while let Some(msg) = conn.next().await? {
            match msg {
                Message::Text(text) => {
                    println!("📨 Received text: {}", text);
                    let echo = format!("Echo: {}", text);
                    conn.send_text(echo).await?;
                    println!("📤 Sent echo response");
                }
                Message::Binary(data) => {
                    println!("📨 Received binary: {} bytes", data.len());
                    conn.send_binary(data).await?;
                    println!("📤 Sent binary echo");
                }
                Message::Ping => {
                    println!("📨 Received ping");
                    conn.pong(None).await?;
                    println!("📤 Sent pong");
                }
                Message::Pong => {
                    println!("📨 Received pong");
                }
                Message::Close(code, reason) => {
                    println!("📨 Received close: {:?} - {:?}", code, reason);
                    break;
                }
            }
        }

        println!("🔌 Connection closed");
        Ok(())
    }).await?;

    Ok(())
}
