//! TLS Transport for AeroSocket
//!
//! This module provides TLS-based transport implementation for secure WebSocket connections.

use aerosocket_core::{
    transport::{Transport, TransportStream},
    Result,
};
use rustls::{ClientConfig, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};

/// TLS transport implementation
pub struct TlsTransport {
    listener: Option<TokioTcpListener>,
    acceptor: Option<TlsAcceptor>,
    local_addr: SocketAddr,
}

impl std::fmt::Debug for TlsTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsTransport")
            .field("local_addr", &self.local_addr)
            .field("has_listener", &self.listener.is_some())
            .field("has_acceptor", &self.acceptor.is_some())
            .finish()
    }
}

impl TlsTransport {
    /// Create a new TLS transport bound to the given address
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(aerosocket_core::Error::Io)?;

        let local_addr = listener.local_addr().map_err(aerosocket_core::Error::Io)?;

        // Create a default server config
        let config = Self::create_server_config()?;
        let acceptor = TlsAcceptor::from(Arc::new(config));

        Ok(Self {
            listener: Some(listener),
            acceptor: Some(acceptor),
            local_addr,
        })
    }

    /// Create a new unbound TLS transport
    pub fn new_unbound() -> Self {
        Self {
            listener: None,
            acceptor: None,
            local_addr: "0.0.0.0:0".parse().unwrap(),
        }
    }

    /// Create a server config with default settings
    fn create_server_config() -> Result<ServerConfig> {
        // For now, create a config that will work for testing
        // In production, users should provide their own certificates
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth();

        // Try to create with empty certificates for testing
        // This will fail in production but allows compilation
        let empty_cert = rustls::Certificate(vec![]);
        let empty_key = rustls::PrivateKey(vec![]);

        match config.with_single_cert(vec![empty_cert], empty_key) {
            Ok(config) => Ok(config),
            Err(_) => {
                // Fallback: create a minimal config for testing
                Err(aerosocket_core::Error::Other(
                    "TLS server configuration requires valid certificates. Use TlsTransport::with_config() to provide custom certificates.".to_string()
                ))
            }
        }
    }

    /// Create a TLS transport with custom server configuration
    pub fn with_config(
        config: ServerConfig,
        addr: SocketAddr,
    ) -> impl std::future::Future<Output = Result<Self>> + Send {
        Box::pin(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(aerosocket_core::Error::Io)?;

            let local_addr = listener.local_addr().map_err(aerosocket_core::Error::Io)?;

            let acceptor = TlsAcceptor::from(Arc::new(config));

            Ok(Self {
                listener: Some(listener),
                acceptor: Some(acceptor),
                local_addr,
            })
        })
    }

    /// Create a client config with default settings
    pub fn create_client_config() -> Result<ClientConfig> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(config)
    }
}

impl Default for TlsTransport {
    fn default() -> Self {
        Self::new_unbound()
    }
}

#[async_trait::async_trait]
impl Transport for TlsTransport {
    type Stream = TlsStream;

    async fn accept(&self) -> Result<Self::Stream> {
        match &self.listener {
            Some(listener) => match &self.acceptor {
                Some(acceptor) => {
                    let (stream, _addr) = listener
                        .accept()
                        .await
                        .map_err(aerosocket_core::Error::Io)?;

                    let tls_stream = acceptor.accept(stream).await.map_err(|e| {
                        aerosocket_core::Error::Other(format!("TLS handshake failed: {}", e))
                    })?;

                    Ok(TlsStream::from_server_tls_stream(tls_stream))
                }
                None => Err(aerosocket_core::Error::Other(
                    "TLS acceptor not configured".to_string(),
                )),
            },
            None => Err(aerosocket_core::Error::Other(
                "Transport not bound".to_string(),
            )),
        }
    }

    fn local_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.local_addr)
    }

    async fn close(self) -> Result<()> {
        // TCP listener is closed automatically when dropped
        Ok(())
    }
}

/// TLS stream implementation
#[derive(Debug)]
pub struct TlsStream {
    stream: Option<tokio_rustls::TlsStream<TokioTcpStream>>,
    remote_addr: SocketAddr,
}

impl TlsStream {
    /// Create a new TLS stream from a tokio-rustls server TLS stream
    pub fn from_server_tls_stream(stream: tokio_rustls::server::TlsStream<TokioTcpStream>) -> Self {
        let remote_addr = stream
            .get_ref()
            .0
            .peer_addr()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

        Self {
            stream: Some(tokio_rustls::TlsStream::Server(stream)),
            remote_addr,
        }
    }

    /// Create a new TLS stream from a tokio-rustls client TLS stream
    pub fn from_client_tls_stream(stream: tokio_rustls::client::TlsStream<TokioTcpStream>) -> Self {
        let remote_addr = stream
            .get_ref()
            .0
            .peer_addr()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

        Self {
            stream: Some(tokio_rustls::TlsStream::Client(stream)),
            remote_addr,
        }
    }

    /// Create a new TLS stream by connecting to a remote address
    pub async fn connect(addr: SocketAddr, config: Arc<ClientConfig>) -> Result<Self> {
        let tcp_stream = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(aerosocket_core::Error::Io)?;

        let connector = TlsConnector::from(config);
        let domain = rustls::ServerName::try_from("localhost")
            .map_err(|e| aerosocket_core::Error::Other(format!("Invalid domain name: {}", e)))?;

        let tls_stream = connector
            .connect(domain, tcp_stream)
            .await
            .map_err(|e| aerosocket_core::Error::Other(format!("TLS connection failed: {}", e)))?;

        Ok(Self::from_client_tls_stream(tls_stream))
    }

    /// Create a new empty TLS stream (for testing)
    pub fn new() -> Self {
        Self {
            stream: None,
            remote_addr: "0.0.0.0:0".parse().unwrap(),
        }
    }
}

impl Default for TlsStream {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TransportStream for TlsStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match &mut self.stream {
            Some(stream) => {
                use tokio::io::AsyncReadExt;
                let n = stream.read(buf).await.map_err(aerosocket_core::Error::Io)?;
                Ok(n)
            }
            None => Err(aerosocket_core::Error::Other(
                "TLS stream not connected".to_string(),
            )),
        }
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match &mut self.stream {
            Some(stream) => {
                use tokio::io::AsyncWriteExt;
                let n = stream
                    .write(buf)
                    .await
                    .map_err(aerosocket_core::Error::Io)?;
                Ok(n)
            }
            None => Err(aerosocket_core::Error::Other(
                "TLS stream not connected".to_string(),
            )),
        }
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match &mut self.stream {
            Some(stream) => {
                use tokio::io::AsyncWriteExt;
                stream
                    .write_all(buf)
                    .await
                    .map_err(aerosocket_core::Error::Io)?;
                Ok(())
            }
            None => Err(aerosocket_core::Error::Other(
                "TLS stream not connected".to_string(),
            )),
        }
    }

    async fn flush(&mut self) -> Result<()> {
        match &mut self.stream {
            Some(stream) => {
                use tokio::io::AsyncWriteExt;
                stream.flush().await.map_err(aerosocket_core::Error::Io)?;
                Ok(())
            }
            None => Err(aerosocket_core::Error::Other(
                "TLS stream not connected".to_string(),
            )),
        }
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            use tokio::io::AsyncWriteExt;
            stream
                .shutdown()
                .await
                .map_err(aerosocket_core::Error::Io)?;
        }
        Ok(())
    }

    fn remote_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.remote_addr)
    }

    fn local_addr(&self) -> Result<std::net::SocketAddr> {
        match &self.stream {
            Some(stream) => match stream {
                tokio_rustls::TlsStream::Server(s) => s
                    .get_ref()
                    .0
                    .local_addr()
                    .map_err(aerosocket_core::Error::Io),
                tokio_rustls::TlsStream::Client(s) => s
                    .get_ref()
                    .0
                    .local_addr()
                    .map_err(aerosocket_core::Error::Io),
            },
            None => Err(aerosocket_core::Error::Other(
                "TLS stream not connected".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_transport_creation() {
        let _transport = TlsTransport::new_unbound();
        // Basic creation test
    }

    #[test]
    fn test_tls_stream_creation() {
        let _stream = TlsStream::new();
        // Basic creation test
    }
}
