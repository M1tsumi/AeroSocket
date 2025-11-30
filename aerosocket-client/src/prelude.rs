//! Prelude module for AeroSocket Client
//!
//! This module re-exports commonly used types and traits to make them
//! easily accessible for users of the client library.

pub use crate::client::{Client, ClientBuilder, ClientConnection};
pub use crate::config::{ClientConfig, CompressionConfig, TlsConfig};
pub use crate::connection::{ClientConnectionHandle, ConnectionMetadata, ConnectionState};

// Re-export core types for convenience
pub use aerosocket_core::prelude::*;
pub use aerosocket_core::{Message, Result};

// Re-export commonly used external dependencies
pub use std::net::SocketAddr;
pub use std::time::Duration;
