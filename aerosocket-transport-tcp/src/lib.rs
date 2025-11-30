//! TCP Transport for AeroSocket
//!
//! This module provides TCP-based transport implementation for WebSocket connections.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket-transport-tcp/")]

pub mod tcp;

// Re-export TCP transport types
pub use tcp::{TcpStream, TcpTransport};

/// Prelude module
pub mod prelude {
    pub use crate::tcp::{TcpStream, TcpTransport};
    pub use aerosocket_core::transport::{Transport, TransportStream};
}
