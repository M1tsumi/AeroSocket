//! Prelude module with common imports
//!
//! This module re-exports the most commonly used types and traits
//! from the aerosocket-server crate for ergonomic imports.

// Server types
pub use crate::config::{
    BackpressureConfig, BackpressureStrategy, CompressionConfig, ServerConfig, TlsConfig,
};
pub use crate::connection::{Connection, ConnectionHandle, ConnectionMetadata, ConnectionState};
pub use crate::handler::{from_fn, BoxedHandler, DefaultHandler, EchoHandler, Handler};
pub use crate::server::{Server, ServerBuilder};

// Re-export core types
pub use aerosocket_core::prelude::*;
