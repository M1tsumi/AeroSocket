//! Prelude module with common imports
//!
//! This module re-exports the most commonly used types and traits
//! from the aerosocket-server crate for ergonomic imports.

// Server types
pub use crate::server::{Server, ServerBuilder};
pub use crate::connection::{Connection, ConnectionHandle, ConnectionState, ConnectionMetadata};
pub use crate::config::{
    ServerConfig, CompressionConfig, BackpressureConfig, BackpressureStrategy, TlsConfig,
};
pub use crate::handler::{Handler, BoxedHandler, DefaultHandler, EchoHandler, from_fn};

// Re-export core types
pub use aerosocket_core::prelude::*;
