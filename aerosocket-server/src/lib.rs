//! AeroSocket Server
//!
//! High-performance WebSocket server implementation with enterprise features.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use aerosocket_server::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> aerosocket_core::Result<()> {
//!     let server = Server::builder()
//!         .bind("0.0.0.0:8080")?
//!         .max_connections(10_000)
//!         .build()?;
//!
//!     server.serve().await?;
//!
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket-server/")]

// Public modules
pub mod config;
pub mod connection;
pub mod error;
pub mod handler;
pub mod logging;
pub mod manager;
pub mod rate_limit;
pub mod server;
pub mod tcp_transport;
pub mod tls_transport;

// Prelude module with common imports
pub mod prelude;

// Re-export key types for convenience
pub use config::{BackpressureConfig, CompressionConfig, ServerConfig, TlsConfig};
pub use connection::{Connection, ConnectionHandle, ConnectionMetadata, ConnectionState};
pub use error::{
    ConfigError, ConnectionError, ContextError, ContextResult, ErrorContext, HandlerError,
    HandshakeError, ManagerError, ProtocolError, ServerError, TransportError,
};
pub use handler::{BoxedHandler, DefaultHandler, EchoHandler, Handler};
pub use manager::{CloseReason, ConnectionHealth, ConnectionManager, ManagerStats};
pub use server::{Server, ServerBuilder};
