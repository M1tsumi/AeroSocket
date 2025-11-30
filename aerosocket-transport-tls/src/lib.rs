//! TLS Transport for AeroSocket
//!
//! This module provides TLS-based transport implementation for secure WebSocket connections.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket-transport-tls/")]

pub mod tls;

// Re-export TLS transport types
pub use tls::{TlsStream, TlsTransport};

/// Prelude module
pub mod prelude {
    pub use crate::tls::{TlsStream, TlsTransport};
    pub use aerosocket_core::transport::{Transport, TransportStream};
}
