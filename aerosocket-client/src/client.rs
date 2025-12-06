//! WebSocket client implementation for AeroSocket
//!
//! This module provides client functionality for WebSocket connections.

use aerosocket_core::{Error, Message, Result};
use aerosocket_core::handshake::{
    create_client_handshake, parse_server_handshake, request_to_string, validate_server_handshake,
    HandshakeConfig,
};
use aerosocket_core::protocol::constants::{HEADER_SEC_WEBSOCKET_KEY, MAX_HEADER_SIZE};
use aerosocket_core::transport::TransportStream;
#[cfg(feature = "transport-tcp")]
use aerosocket_transport_tcp::TcpStream;
#[cfg(feature = "transport-tls")]
use aerosocket_transport_tls::TlsStream;
use crate::config::ClientConfig as ClientOptions;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::time::timeout;
#[cfg(feature = "transport-tls")]
use std::sync::Arc;

/// WebSocket client
#[derive(Debug)]
pub struct Client {
    /// Server address
    addr: SocketAddr,
    /// Client configuration
    config: ClientOptions,
}

impl Client {
    /// Create a new client
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            config: ClientOptions::default(),
        }
    }

    /// Set client configuration
    pub fn with_config(mut self, config: ClientOptions) -> Self {
        self.config = config;
        self
    }

    /// Connect to the WebSocket server
    #[cfg(any(feature = "transport-tcp", feature = "transport-tls"))]
    #[cfg_attr(feature = "logging", tracing::instrument(skip(self)))]
    pub async fn connect(self) -> Result<crate::connection::ClientConnection> {
        let addr = self.addr;
        let config = self.config.clone();
        let handshake_timeout = config.handshake_timeout;

        let fut = async move {
            #[cfg(feature = "metrics")]
            let handshake_start = Instant::now();

            // Build handshake config from client settings
            let mut handshake_config = HandshakeConfig::default();
            handshake_config.protocols = config.protocols.clone();
            if let Some(origin) = &config.origin {
                handshake_config.origin = Some(origin.clone());
            }
            for (name, value) in &config.headers {
                handshake_config
                    .extra_headers
                    .insert(name.clone(), value.clone());
            }

            // Decide between TLS and TCP based on TLS configuration
            if let Some(tls_cfg) = &config.tls {
                #[cfg(feature = "transport-tls")]
                {
                    let server_name = tls_cfg
                        .server_name
                        .as_deref()
                        .unwrap_or("localhost");

                    handshake_config.host = Some(format!("{}:{}", server_name, addr.port()));
                    let uri = format!("wss://{}:{}", server_name, addr.port());
                    let request = create_client_handshake(&uri, &handshake_config)?;

                    let client_key = request
                        .headers
                        .get(HEADER_SEC_WEBSOCKET_KEY)
                        .cloned()
                        .ok_or_else(|| {
                            Error::Other(
                                "Missing sec-websocket-key in client handshake".to_string(),
                            )
                        })?;

                    let request_string = request_to_string(&request);

                    let tls_config = crate::config::build_rustls_client_config(tls_cfg)?;
                    let mut stream = TlsStream::connect(addr, Arc::new(tls_config), server_name)
                        .await?;

                    stream.write_all(request_string.as_bytes()).await?;
                    stream.flush().await?;

                    let mut buffer = Vec::new();
                    let mut temp = [0u8; 1024];

                    loop {
                        let n = stream.read(&mut temp).await?;
                        if n == 0 {
                            break;
                        }
                        buffer.extend_from_slice(&temp[..n]);
                        if buffer.len() > MAX_HEADER_SIZE {
                            return Err(Error::Other("Server handshake too large".to_string()));
                        }
                        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }

                    let raw_response = String::from_utf8_lossy(&buffer).to_string();
                    let response = parse_server_handshake(&raw_response)?;
                    validate_server_handshake(&response, &client_key)?;

                    let remote_addr = stream.remote_addr()?;
                    let mut connection = crate::connection::ClientConnection::with_stream(
                        remote_addr,
                        Box::new(stream) as Box<dyn TransportStream>,
                    );
                    connection.set_connected();

                    #[cfg(feature = "metrics")]
                    {
                        let elapsed = handshake_start.elapsed().as_secs_f64();
                        metrics::histogram!("aerosocket_client_handshake_duration_seconds")
                            .record(elapsed);
                        metrics::counter!("aerosocket_client_connections_opened_total")
                            .increment(1);
                    }

                    Ok(connection)
                }

                #[cfg(not(feature = "transport-tls"))]
                {
                    let _ = tls_cfg;
                    Err(Error::Other(
                        "TLS configuration provided but transport-tls feature is not enabled for aerosocket-client"
                            .to_string(),
                    ))
                }
            } else {
                #[cfg(feature = "transport-tcp")]
                {
                    let uri = format!("ws://{}", addr);
                    let request = create_client_handshake(&uri, &handshake_config)?;

                    let client_key = request
                        .headers
                        .get(HEADER_SEC_WEBSOCKET_KEY)
                        .cloned()
                        .ok_or_else(|| {
                            Error::Other(
                                "Missing sec-websocket-key in client handshake".to_string(),
                            )
                        })?;

                    let request_string = request_to_string(&request);
                    let mut stream = TcpStream::connect(addr).await?;

                    stream.write_all(request_string.as_bytes()).await?;
                    stream.flush().await?;

                    let mut buffer = Vec::new();
                    let mut temp = [0u8; 1024];

                    loop {
                        let n = stream.read(&mut temp).await?;
                        if n == 0 {
                            break;
                        }
                        buffer.extend_from_slice(&temp[..n]);
                        if buffer.len() > MAX_HEADER_SIZE {
                            return Err(Error::Other("Server handshake too large".to_string()));
                        }
                        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }

                    let raw_response = String::from_utf8_lossy(&buffer).to_string();
                    let response = parse_server_handshake(&raw_response)?;
                    validate_server_handshake(&response, &client_key)?;

                    let remote_addr = stream.remote_addr()?;
                    let mut connection = crate::connection::ClientConnection::with_stream(
                        remote_addr,
                        Box::new(stream) as Box<dyn TransportStream>,
                    );
                    connection.set_connected();

                    #[cfg(feature = "metrics")]
                    {
                        let elapsed = handshake_start.elapsed().as_secs_f64();
                        metrics::histogram!("aerosocket_client_handshake_duration_seconds")
                            .record(elapsed);
                        metrics::counter!("aerosocket_client_connections_opened_total")
                            .increment(1);
                    }

                    Ok(connection)
                }

                #[cfg(not(feature = "transport-tcp"))]
                {
                    Err(Error::Other(
                        "No transport feature enabled (transport-tcp or transport-tls) for aerosocket-client"
                            .to_string(),
                    ))
                }
            }
        };

        match timeout(handshake_timeout, fut).await {
            Ok(result) => result,
            Err(_) => Err(Error::Timeout(aerosocket_core::error::TimeoutError::Handshake {
                timeout: handshake_timeout,
            })),
        }
    }

    /// Connect to the WebSocket server (requires a transport feature)
    #[cfg(not(any(feature = "transport-tcp", feature = "transport-tls")))]
    pub async fn connect(self) -> Result<crate::connection::ClientConnection> {
        Err(Error::Other(
            "No transport feature enabled (transport-tcp or transport-tls) for aerosocket-client"
                .to_string(),
        ))
    }
}

/// Client connection
#[derive(Debug)]
pub struct ClientConnection {
    /// Server address
    #[allow(dead_code)]
    remote_addr: SocketAddr,
    /// Connection state
    connected: bool,
}

impl ClientConnection {
    /// Create a new client connection
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            remote_addr: addr,
            connected: false,
        }
    }

    /// Send a message
    pub async fn send(&mut self, _message: Message) -> Result<()> {
        // TODO: Implement actual message sending
        Ok(())
    }

    /// Receive the next message
    pub async fn next(&mut self) -> Result<Option<Message>> {
        // TODO: Implement actual message receiving
        Ok(None)
    }

    /// Close the connection
    pub async fn close(&mut self, _code: Option<u16>, _reason: Option<&str>) -> Result<()> {
        // TODO: Implement actual connection closing
        self.connected = false;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

/// Client builder
#[derive(Debug)]
pub struct ClientBuilder {
    addr: SocketAddr,
    config: ClientOptions,
}

impl ClientBuilder {
    /// Create a new client builder
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            config: ClientOptions::default(),
        }
    }

    /// Set maximum frame size
    pub fn max_frame_size(mut self, size: usize) -> Self {
        self.config.max_frame_size = size;
        self
    }

    /// Set maximum message size
    pub fn max_message_size(mut self, size: usize) -> Self {
        self.config.max_message_size = size;
        self
    }

    /// Set handshake timeout
    pub fn handshake_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.handshake_timeout = timeout;
        self
    }

    /// Enable/disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.config.compression.enabled = enabled;
        self
    }

    /// Build the client
    pub fn build(self) -> Client {
        Client::new(self.addr).with_config(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let client = Client::new(addr);
        assert_eq!(client.addr, addr);
    }

    #[test]
    fn test_client_config() {
        let config = ClientConfig::default();
        assert_eq!(config.max_frame_size, 1024 * 1024);
        assert_eq!(config.max_message_size, 16 * 1024 * 1024);
    }

    #[test]
    fn test_client_builder() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let client = ClientBuilder::new(addr)
            .max_frame_size(2048)
            .compression(true)
            .build();

        assert_eq!(client.addr, addr);
        assert_eq!(client.config.max_frame_size, 2048);
        assert!(client.config.compression_enabled);
    }
}
