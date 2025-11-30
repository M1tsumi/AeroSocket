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
//! use aerosocket::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server = aerosocket::Server::builder()
//!         .bind("0.0.0.0:8080")
//!         .max_connections(10_000)
//!         .build()?;
//!
//!     server.serve(|mut conn| async move {
//!         while let Some(msg) = conn.next().await? {
//!             match msg {
//!                 Message::Text(text) => conn.send_text(text).await?,
//!                 Message::Binary(data) => conn.send_binary(data).await?,
//!                 Message::Ping => conn.send_pong().await?,
//!                 _ => {}
//!             }
//!         }
//!         Ok(())
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/aerosocket/")]

// Re-export core components
pub use aerosocket_core::*;

#[cfg(feature = "transport-tcp")]
pub use aerosocket_transport_tcp as transport_tcp;

#[cfg(feature = "transport-tls")]
pub use aerosocket_transport_tls as transport_tls;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_compiles() {
        // Basic test to ensure the library compiles correctly
        assert_eq!(env!("CARGO_PKG_NAME"), "aerosocket");
    }
}
