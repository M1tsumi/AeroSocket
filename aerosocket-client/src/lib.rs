//! AeroSocket Client
//!
//! High-performance WebSocket client implementation with enterprise features.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use aerosocket_client::prelude::*;
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> aerosocket_core::Result<()> {
//!     let addr: SocketAddr = "127.0.0.1:8080".parse()
//!         .map_err(|e| aerosocket_core::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;
//!     let mut client = aerosocket_client::ClientConnection::new(addr);
//!
//!     // Note: This is a simplified example - actual connection logic would be implemented here
//!     println!("Client created successfully for {}", addr);
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
pub use config::{ClientConfig, CompressionConfig, TlsConfig};
pub use connection::ClientConnection;
