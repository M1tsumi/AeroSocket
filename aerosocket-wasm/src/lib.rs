//! WebAssembly support for AeroSocket
//!
//! This module provides WebSocket client functionality for WebAssembly targets,
//! enabling AeroSocket to run in browsers and WASM environments.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket-wasm/")]

#[cfg(feature = "wasm-bindgen")]
pub mod wasm;

#[cfg(feature = "wasm-bindgen")]
pub use wasm::{WebSocketClient, WebSocketConfig};

/// Prelude module
pub mod prelude {
    #[cfg(feature = "wasm-bindgen")]
    pub use crate::wasm::{WebSocketClient, WebSocketConfig};
    pub use aerosocket_core::prelude::*;
}
