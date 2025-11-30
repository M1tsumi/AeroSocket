//! Simple WebSocket server example
//!
//! This example demonstrates how to create a basic WebSocket server
//! using the AeroSocket library.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple echo server
    let server = aerosocket_server::ServerBuilder::new()
        .bind("127.0.0.1:8080")?
        .max_connections(1000)
        .handshake_timeout(std::time::Duration::from_secs(10))
        .build()?;

    println!("WebSocket server listening on ws://127.0.0.1:8080");
    
    // Start the server
    server.serve().await?;
    
    Ok(())
}
