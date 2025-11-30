//! AeroSocket Client
//!
//! High-performance WebSocket client implementation with enterprise features.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use aerosocket_client::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = Client::connect("wss://echo.websocket.org")
//!         .with_header("Authorization", "Bearer token")
//!         .connect()
//!         .await?;
//!
//!     client.send_text("Hello, AeroSocket!").await?;
//!
//!     while let Some(msg) = client.next().await? {
//!         println!("Received: {:?}", msg);
//!         break;
//!     }
//!
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket-client/")]

// Public modules
pub mod client;
pub mod config;
pub mod connection;

// Prelude module
pub mod prelude;

// Re-export key types for convenience
pub use client::{Client, ClientBuilder};
pub use config::{ClientConfig, TlsConfig, CompressionConfig};
pub use connection::ClientConnection;
