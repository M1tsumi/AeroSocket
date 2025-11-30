//! TLS transport implementation for WebSocket server
//!
//! This module provides TLS/SSL support for secure WebSocket connections.
//! Note: TLS functionality requires the "tls-transport" feature and proper certificate setup.

#[cfg(feature = "tls-transport")]
use aerosocket_core::{Error, Result, Transport};
#[cfg(feature = "tls-transport")]
use aerosocket_core::transport::TransportStream;
#[cfg(feature = "tls-transport")]
use async_trait::async_trait;
#[cfg(feature = "tls-transport")]
use std::net::SocketAddr;
#[cfg(feature = "tls-transport")]
use std::sync::Arc;

#[cfg(feature = "tls-transport")]
use tokio::net::{TcpListener, TcpStream as TokioTcpStream};
#[cfg(feature = "tls-transport")]
use tokio_rustls::{TlsAcceptor, server::TlsStream, rustls::ServerConfig as RustlsServerConfig};

#[cfg(feature = "tls-transport")]
/// TLS transport for WebSocket connections
pub struct TlsTransport {
    /// TCP listener
    listener: TcpListener,
    /// TLS acceptor
    acceptor: TlsAcceptor,
    /// Local address
    local_addr: SocketAddr,
}

#[cfg(feature = "tls-transport")]
/// TLS stream wrapper
pub struct TlsStreamWrapper {
    inner: TlsStream<TokioTcpStream>,
}

#[cfg(feature = "tls-transport")]
#[async_trait]
impl Transport for TlsTransport {
    type Stream = TlsStreamWrapper;

    async fn accept(&self) -> Result<Self::Stream> {
        let tcp_stream = self.listener.accept().await
            .map_err(|e| Error::Io(e))?.0;
        
        let tls_stream = self.acceptor.accept(tcp_stream).await
            .map_err(|e| Error::Other(format!("Failed to accept TLS connection: {}", e)))?;

        Ok(TlsStreamWrapper { inner: tls_stream })
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.local_addr)
    }

    async fn close(self) -> Result<()> {
        // The listener will be closed when dropped
        Ok(())
    }
}

#[cfg(feature = "tls-transport")]
#[async_trait]
impl TransportStream for TlsStreamWrapper {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        use tokio::io::AsyncReadExt;
        self.inner.read(buf).await
            .map_err(|e| Error::Io(e))
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        use tokio::io::AsyncWriteExt;
        self.inner.write(buf).await
            .map_err(|e| Error::Io(e))
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        self.inner.write_all(buf).await
            .map_err(|e| Error::Io(e))
    }

    async fn flush(&mut self) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        self.inner.flush().await
            .map_err(|e| Error::Io(e))
    }

    async fn close(&mut self) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        self.inner.shutdown().await
            .map_err(|e| Error::Io(e))
    }

    fn remote_addr(&self) -> Result<SocketAddr> {
        self.inner.get_ref().0.peer_addr()
            .map_err(|e| Error::Io(e))
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        self.inner.get_ref().0.local_addr()
            .map_err(|e| Error::Io(e))
    }
}

#[cfg(feature = "tls-transport")]
impl TlsTransport {
    /// Bind to the given address with TLS configuration
    pub async fn bind(addr: SocketAddr, tls_config: RustlsServerConfig) -> Result<Self> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| Error::Io(e))?;
        
        let local_addr = listener.local_addr()
            .map_err(|e| Error::Io(e))?;

        let acceptor = TlsAcceptor::from(Arc::new(tls_config));

        Ok(Self {
            listener,
            acceptor,
            local_addr,
        })
    }

    /// Create a new TLS transport with default configuration
    pub async fn bind_with_default_config(addr: SocketAddr) -> Result<Self> {
        let config = create_default_tls_config()?;
        Self::bind(addr, config).await
    }
}

#[cfg(feature = "tls-transport")]
/// Create a default TLS configuration for testing/development
pub fn create_default_tls_config() -> Result<RustlsServerConfig> {
    // For now, return a basic config - in production this should load real certificates
    Err(Error::Other("TLS configuration not available in this release. Please implement your own TLS config.".to_string()))
}

#[cfg(not(feature = "tls-transport"))]
/// TLS transport is not available without the tls-transport feature
pub struct TlsTransport;

#[cfg(not(feature = "tls-transport"))]
impl TlsTransport {
    pub async fn bind(_addr: std::net::SocketAddr, _config: ()) -> Result<Self> {
        Err(Error::Other("TLS transport requires the 'tls-transport' feature to be enabled".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tls_transport_creation() {
        // This test requires certificates to be present
        // In a real scenario, you would have proper test certificates
        let addr = "127.0.0.1:0".parse().unwrap();
        
        // Skip test if certificates are not available
        if let Ok(config) = create_default_tls_config() {
            let result = TlsTransport::bind(addr, config).await;
            assert!(result.is_ok());
        }
    }
}
