//! TCP Transport for AeroSocket
//!
//! This module provides TCP-based transport implementation for WebSocket connections.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket-transport-tcp/")]

pub mod tcp;

// Re-export TCP transport types
pub use tcp::{TcpTransport, TcpStream};

/// Prelude module
pub mod prelude {
    pub use crate::tcp::{TcpTransport, TcpStream};
    pub use aerosocket_core::transport::{Transport, TransportStream};
}
