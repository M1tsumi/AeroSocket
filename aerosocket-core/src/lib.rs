//! # AeroSocket Core
//!
//! Core WebSocket protocol implementation providing the foundation for AeroSocket Core Library
//!
//! This is the core library that provides the fundamental WebSocket protocol
//! implementation for AeroSocket. It includes:
//!
//! - Error handling and types
//! - WebSocket frame parsing and generation
//! - Message handling and assembly
//! - Protocol constants and utilities
//! - Transport layer abstractions

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket-core/")]

// Core modules
pub mod error;
pub mod frame;
pub mod handshake;
pub mod message;
pub mod protocol;
pub mod transport;

// Prelude module with common imports
pub mod prelude;

// Re-export key types for convenience
pub use error::{Error, Result};
pub use frame::{Frame, FrameKind};
pub use handshake::{HandshakeConfig, HandshakeRequest, HandshakeResponse};
pub use message::{Message, MessageKind};
pub use protocol::Opcode;
pub use transport::Transport;
