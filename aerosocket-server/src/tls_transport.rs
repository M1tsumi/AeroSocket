//! TLS transport implementation for WebSocket server
//!
//! This module provides TLS/SSL support for secure WebSocket connections.

use aerosocket_core::{Error, Result, Transport};
use aerosocket_core::transport::TransportStream;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream as TokioTcpStream};
use tokio_rustls::{TlsAcceptor, server::TlsStream, rustls::ServerConfig as RustlsServerConfig};

/// TLS transport for WebSocket connections
pub struct TlsTransport {
    /// TCP listener
    listener: TcpListener,
    /// TLS acceptor
    acceptor: TlsAcceptor,
    /// Local address
    local_addr: SocketAddr,
}

/// TLS stream wrapper
pub struct TlsStreamWrapper {
    inner: TlsStream<TokioTcpStream>,
}

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

/// Create a default TLS configuration for testing/development
pub fn create_default_tls_config() -> Result<RustlsServerConfig> {
    use rustls::Certificate;
    use rustls::PrivateKey;
    use rustls::ServerConfig;
    use std::io::Cursor;

    // For development purposes, we'll create a self-signed certificate
    // In production, you should load proper certificates from files
    let cert_pem = include_str!("../certificates/server.crt");
    let key_pem = include_str!("../certificates/server.key");

    let cert_der = rustls_pemfile::certs(&mut Cursor::new(cert_pem))
        .into_iter()
        .flatten()
        .map(Certificate)
        .collect();

    let key_der = rustls_pemfile::pkcs8_private_keys(&mut Cursor::new(key_pem))
        .into_iter()
        .flatten()
        .map(PrivateKey)
        .next()
        .ok_or_else(|| Error::Other("No private key found".to_string()))?;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_der, key_der)
        .map_err(|e| Error::Other(format!("Failed to create TLS config: {}", e)))?;

    Ok(config)
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
