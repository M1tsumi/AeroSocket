//! Echo Client Example
//!
//! This example demonstrates a WebSocket client that connects to an echo server
//! and sends messages to test the connection.

use aerosocket::prelude::*;

#[cfg(feature = "client")]
use aerosocket::client::{Client, ClientConfig};
#[cfg(feature = "client")]
use std::net::SocketAddr;

#[cfg(feature = "client")]
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create and connect the client
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let config = ClientConfig::default();
    let client = Client::new(addr).with_config(config);
    let mut client = client.connect().await?;

    println!("🔗 Connected to echo server");

    // Send a text message
    client.send_text("Hello, AeroSocket!").await?;
    println!("📤 Sent: Hello, AeroSocket!");

    // Receive the echo response
    if let Some(msg) = client.next().await? {
        match msg {
            Message::Text(text) => {
                println!("📨 Received: {}", text.as_str());
            }
            _ => {
                println!("📨 Received unexpected message type");
            }
        }
    }

    // Send a binary message
    let binary_data: &[u8] = b"Binary payload";
    client.send_binary(binary_data).await?;
    println!("📤 Sent binary: {} bytes", binary_data.len());

    // Receive the binary echo
    if let Some(msg) = client.next().await? {
        match msg {
            Message::Binary(data) => {
                println!("📨 Received binary: {} bytes", data.len());
            }
            _ => {
                println!("📨 Received unexpected message type");
            }
        }
    }

    // Send a ping
    client.ping(None).await?;
    println!("📤 Sent ping");

    // Wait for pong response
    if let Some(msg) = client.next().await? {
        match msg {
            Message::Pong(_) => {
                println!("📨 Received pong");
            }
            _ => {
                println!("📨 Received unexpected message type");
            }
        }
    }

    // Close the connection
    client.close(Some(1000), Some("Goodbye")).await?;
    println!("🔌 Connection closed");

    Ok(())
}

#[cfg(not(feature = "client"))]
fn main() {
    println!("Enable the 'client' feature to run this example");
}
