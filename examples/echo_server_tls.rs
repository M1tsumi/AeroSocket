//! TLS Echo Server Example
//!
//! This example demonstrates a simple TLS-enabled WebSocket echo server
//! using the aggregated `aerosocket` crate. It requires the `server` and
//! `transport-tls` features to be enabled.

use aerosocket::prelude::*;

#[cfg(all(feature = "server", feature = "transport-tls"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // NOTE: For local testing, generate a self-signed certificate/key pair
    // and point these paths at those files (e.g. "certs/server.crt", "certs/server.key").
    let cert_file = "server.crt";
    let key_file = "server.key";

    // Create the TLS-enabled server
    let server = Server::builder()
        .bind("127.0.0.1:8443")?
        .max_connections(1000)
        .compression(true)
        .tls(cert_file, key_file)
        .transport_tls()
        .build_with_handler(EchoHandler::new())?;

    println!("🚀 TLS echo server listening on wss://127.0.0.1:8443");

    server.serve().await?;

    Ok(())
}

#[cfg(not(all(feature = "server", feature = "transport-tls")))]
fn main() {
    eprintln!("Enable the 'server' and 'transport-tls' features on the 'aerosocket' crate to run this example.");
}
