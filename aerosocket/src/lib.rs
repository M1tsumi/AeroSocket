//! # 🚀 AeroSocket
//!
//! **Ultra-fast, zero-copy WebSocket library for enterprise-scale applications**
//!
//! AeroSocket delivers exceptional performance and reliability for real-time applications.
//! Built with a focus on zero-copy operations, enterprise stability, and developer ergonomics.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! #[cfg(feature = "server")]
//! use aerosocket::prelude::*;
//!
//! #[cfg(feature = "server")]
//! #[tokio::main]
//! async fn main() -> aerosocket_core::Result<()> {
//!     let server = aerosocket::server::Server::builder()
//!         .max_connections(10_000)
//!         .build()?;
//!
//!     server.serve().await?;
//!
//!     Ok(())
//! }
//!
//! #[cfg(not(feature = "server"))]
//! fn main() {
//!     println!("Enable the 'server' feature to run this example");
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(ambiguous_glob_reexports)]
#![warn(rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket/")]

// Re-export core components
pub use aerosocket_core::*;

#[cfg(feature = "transport-tcp")]
pub use aerosocket_transport_tcp as transport_tcp;

#[cfg(feature = "transport-tls")]
pub use aerosocket_transport_tls as transport_tls;

#[cfg(feature = "wasm")]
pub use aerosocket_wasm as wasm;

#[cfg(feature = "server")]
pub use aerosocket_server as server;

#[cfg(feature = "client")]
pub use aerosocket_client as client;

/// Prelude module with common imports
pub mod prelude {
    pub use aerosocket_core::prelude::*;

    #[cfg(feature = "server")]
    pub use aerosocket_server::prelude::*;

    #[cfg(feature = "client")]
    pub use aerosocket_client::prelude::*;

    #[cfg(feature = "transport-tcp")]
    pub use aerosocket_transport_tcp::prelude::*;

    #[cfg(feature = "transport-tls")]
    pub use aerosocket_transport_tls::prelude::*;

    #[cfg(feature = "wasm")]
    pub use aerosocket_wasm::prelude::*;
}

#[cfg(test)]
mod tests {
    // Test module placeholder
    #[test]
    fn test_library_compiles() {
        // Basic test to ensure the library compiles correctly
        assert_eq!(env!("CARGO_PKG_NAME"), "aerosocket");
    }
}
