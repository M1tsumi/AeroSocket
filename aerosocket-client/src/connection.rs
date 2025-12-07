//! WebSocket client connection handling for AeroSocket
//!
//! This module provides connection management for WebSocket clients.

use aerosocket_core::frame::Frame;
use aerosocket_core::protocol::Opcode;
use aerosocket_core::transport::TransportStream;
use aerosocket_core::{Message, Result};
use bytes::{Bytes, BytesMut};
use std::fmt;
use std::net::SocketAddr;

/// Represents a WebSocket client connection
pub struct ClientConnection {
    /// Server address
    remote_addr: SocketAddr,
    /// Connection state
    state: ConnectionState,
    /// Connection metadata
    metadata: ConnectionMetadata,
    stream: Option<Box<dyn TransportStream>>,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    /// Connection is established and ready
    Connected,
    /// Connection is closing
    Closing,
    /// Connection is closed
    Closed,
}

/// Connection metadata
#[derive(Debug, Clone)]
pub struct ConnectionMetadata {
    /// WebSocket subprotocol
    pub subprotocol: Option<String>,
    /// WebSocket extensions
    pub extensions: Vec<String>,
    /// Connection established time
    pub established_at: std::time::Instant,
    /// Last activity time
    pub last_activity_at: std::time::Instant,
    /// Messages sent count
    pub messages_sent: u64,
    /// Messages received count
    pub messages_received: u64,
    /// Bytes sent count
    pub bytes_sent: u64,
    /// Bytes received count
    pub bytes_received: u64,
}

impl ClientConnection {
    /// Create a new client connection
    pub fn new(remote_addr: SocketAddr) -> Self {
        Self {
            remote_addr,
            state: ConnectionState::Connecting,
            metadata: ConnectionMetadata {
                subprotocol: None,
                extensions: Vec::new(),
                established_at: std::time::Instant::now(),
                last_activity_at: std::time::Instant::now(),
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
            },
            stream: None,
        }
    }

    #[allow(missing_docs)]
    pub fn with_stream(remote_addr: SocketAddr, stream: Box<dyn TransportStream>) -> Self {
        let now = std::time::Instant::now();
        Self {
            remote_addr,
            state: ConnectionState::Connected,
            metadata: ConnectionMetadata {
                subprotocol: None,
                extensions: Vec::new(),
                established_at: now,
                last_activity_at: now,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
            },
            stream: Some(stream),
        }
    }

    /// Get the remote address
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Get the connection state
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Get the connection metadata
    pub fn metadata(&self) -> &ConnectionMetadata {
        &self.metadata
    }

    fn update_activity(&mut self) {
        let now = std::time::Instant::now();
        self.metadata.last_activity_at = now;
    }

    /// Send a message
    #[cfg_attr(feature = "logging", tracing::instrument(skip(self, message)))]
    pub async fn send(&mut self, message: Message) -> Result<()> {
        self.update_activity();

        if let Some(stream) = &mut self.stream {
            let frame = match message {
                Message::Text(text) => Frame::text(text.as_bytes().to_vec()),
                Message::Binary(data) => Frame::binary(data.as_bytes().to_vec()),
                Message::Ping(data) => Frame::ping(data.as_bytes().to_vec()),
                Message::Pong(data) => Frame::pong(data.as_bytes().to_vec()),
                Message::Close(close_msg) => {
                    Frame::close(close_msg.code(), Some(close_msg.reason()))
                }
            };

            let frame_bytes = frame.to_bytes();

            #[cfg(feature = "metrics")]
            {
                metrics::counter!("aerosocket_client_messages_sent_total")
                    .increment(1);
                metrics::counter!("aerosocket_client_bytes_sent_total")
                    .increment(frame_bytes.len() as u64);
                metrics::histogram!("aerosocket_client_frame_size_bytes")
                    .record(frame_bytes.len() as f64);
            }

            stream.write_all(&frame_bytes).await?;
            stream.flush().await?;

            self.metadata.messages_sent += 1;
            self.metadata.bytes_sent += frame_bytes.len() as u64;
            self.metadata.last_activity_at = std::time::Instant::now();

            Ok(())
        } else {
            Err(aerosocket_core::Error::Other(
                "Connection not established".to_string(),
            ))
        }
    }

    /// Send a text message
    pub async fn send_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        let message = Message::text(text.as_ref().to_string());
        self.send(message).await
    }

    /// Send a binary message
    pub async fn send_binary(&mut self, data: impl Into<Bytes>) -> Result<()> {
        let message = Message::binary(data);
        self.send(message).await
    }

    /// Send a ping message
    pub async fn ping(&mut self, data: Option<&[u8]>) -> Result<()> {
        let message = Message::ping(data.map(|d| d.to_vec()));
        self.send(message).await
    }

    /// Send a pong message
    pub async fn pong(&mut self, data: Option<&[u8]>) -> Result<()> {
        let message = Message::pong(data.map(|d| d.to_vec()));
        self.send(message).await
    }

    /// Receive the next message
    #[cfg_attr(feature = "logging", tracing::instrument(skip(self)))]
    pub async fn next(&mut self) -> Result<Option<Message>> {
        self.update_activity();

        if let Some(stream) = &mut self.stream {
            let mut message_buffer = Vec::new();
            let mut final_frame = false;
            let mut opcode = None;

            while !final_frame {
                let mut frame_buffer = BytesMut::new();

                loop {
                    let mut temp_buf = [0u8; 2];
                    let n = stream.read(&mut temp_buf).await?;
                    if n == 0 {
                        self.state = ConnectionState::Closed;
                        return Ok(None);
                    }
                    frame_buffer.extend_from_slice(&temp_buf[..n]);

                    if frame_buffer.len() >= 2 {
                        break;
                    }
                }

                match Frame::parse(&mut frame_buffer) {
                    Ok(frame) => {
                        match frame.opcode {
                            Opcode::Ping => {
                                let ping_data = frame.payload.to_vec();
                                stream.write_all(&Frame::pong(ping_data).to_bytes()).await?;
                                stream.flush().await?;
                                continue;
                            }
                            Opcode::Pong => {
                                continue;
                            }
                            Opcode::Close => {
                                let close_code = if frame.payload.len() >= 2 {
                                    let code_bytes = &frame.payload[..2];
                                    u16::from_be_bytes([code_bytes[0], code_bytes[1]])
                                } else {
                                    1000
                                };

                                let close_reason = if frame.payload.len() > 2 {
                                    String::from_utf8_lossy(&frame.payload[2..]).to_string()
                                } else {
                                    String::new()
                                };

                                self.state = ConnectionState::Closing;
                                return Ok(Some(Message::close(
                                    Some(close_code),
                                    Some(close_reason),
                                )));
                            }
                            Opcode::Continuation | Opcode::Text | Opcode::Binary => {
                                if opcode.is_none() {
                                    opcode = Some(frame.opcode);
                                }

                                message_buffer.extend_from_slice(&frame.payload);
                                final_frame = frame.fin;

                                if !final_frame && frame.opcode != Opcode::Continuation {
                                    return Err(aerosocket_core::Error::Other(
                                        "Expected continuation frame".to_string(),
                                    ));
                                }
                            }
                            _ => {
                                return Err(aerosocket_core::Error::Other(
                                    "Unsupported opcode".to_string(),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        let mut temp_buf = [0u8; 1024];
                        match stream.read(&mut temp_buf).await {
                            Ok(0) => {
                                self.state = ConnectionState::Closed;
                                return Ok(None);
                            }
                            Ok(n) => {
                                let _ = e;
                                frame_buffer.extend_from_slice(&temp_buf[..n]);
                            }
                            Err(err) => return Err(err),
                        }
                        continue;
                    }
                }
            }

            let message = match opcode.unwrap_or(Opcode::Text) {
                Opcode::Text => {
                    let text = String::from_utf8_lossy(&message_buffer).to_string();
                    Message::text(text)
                }
                Opcode::Binary => {
                    let data = Bytes::from(message_buffer.clone());
                    Message::binary(data)
                }
                _ => {
                    return Err(aerosocket_core::Error::Other(
                        "Invalid message opcode".to_string(),
                    ));
                }
            };

            self.metadata.messages_received += 1;
            self.metadata.bytes_received += message_buffer.len() as u64;

            #[cfg(feature = "metrics")]
            {
                metrics::counter!("aerosocket_client_messages_received_total")
                    .increment(1);
                metrics::counter!("aerosocket_client_bytes_received_total")
                    .increment(message_buffer.len() as u64);
                metrics::histogram!("aerosocket_client_message_size_bytes")
                    .record(message_buffer.len() as f64);
            }

            Ok(Some(message))
        } else {
            Err(aerosocket_core::Error::Other(
                "Connection not established".to_string(),
            ))
        }
    }

    /// Close the connection
    #[cfg_attr(feature = "logging", tracing::instrument(skip(self, reason)))]
    pub async fn close(&mut self, code: Option<u16>, reason: Option<&str>) -> Result<()> {
        self.state = ConnectionState::Closing;

        #[cfg(feature = "metrics")]
        {
            metrics::counter!("aerosocket_client_connections_closed_total")
                .increment(1);
        }
        let message = Message::close(code, reason.map(|s| s.to_string()));
        self.send(message).await?;
        self.state = ConnectionState::Closed;
        Ok(())
    }

    /// Check if the connection is established
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    /// Check if the connection is closed
    pub fn is_closed(&self) -> bool {
        self.state == ConnectionState::Closed
    }

    /// Get the connection age
    pub fn age(&self) -> std::time::Duration {
        self.metadata.established_at.elapsed()
    }

    /// Get the time since last activity
    pub fn idle_time(&self) -> std::time::Duration {
        self.metadata.last_activity_at.elapsed()
    }

    /// Set the connection as connected
    pub fn set_connected(&mut self) {
        self.state = ConnectionState::Connected;
        self.metadata.established_at = std::time::Instant::now();
        self.metadata.last_activity_at = std::time::Instant::now();
    }

    /// Set the subprotocol
    pub fn set_subprotocol(&mut self, subprotocol: String) {
        self.metadata.subprotocol = Some(subprotocol);
    }

    /// Add an extension
    pub fn add_extension(&mut self, extension: String) {
        self.metadata.extensions.push(extension);
    }
}

/// Connection handle for managing connections
pub struct ClientConnectionHandle {
    /// Connection ID
    id: u64,
    /// Connection reference
    connection: std::sync::Arc<std::sync::Mutex<ClientConnection>>,
}

impl ClientConnectionHandle {
    /// Create a new connection handle
    pub fn new(id: u64, connection: ClientConnection) -> Self {
        Self {
            id,
            connection: std::sync::Arc::new(std::sync::Mutex::new(connection)),
        }
    }

    /// Get the connection ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get a reference to the connection
    pub fn connection(&self) -> &std::sync::Arc<std::sync::Mutex<ClientConnection>> {
        &self.connection
    }

    /// Try to lock the connection
    pub fn try_lock(&self) -> aerosocket_core::Result<std::sync::MutexGuard<'_, ClientConnection>> {
        self.connection
            .try_lock()
            .map_err(|e| aerosocket_core::Error::Other(format!("Poison error: {}", e)))
    }
}

impl Clone for ClientConnectionHandle {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            connection: self.connection.clone(),
        }
    }
}

impl fmt::Debug for ClientConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientConnection")
            .field("remote_addr", &self.remote_addr)
            .field("state", &self.state)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl fmt::Debug for ClientConnectionHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientConnectionHandle")
            .field("id", &self.id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_connection_creation() {
        let remote = "127.0.0.1:8080".parse().unwrap();
        let conn = ClientConnection::new(remote);

        assert_eq!(conn.remote_addr(), remote);
        assert_eq!(conn.state(), ConnectionState::Connecting);
        assert!(!conn.is_connected());
        assert!(!conn.is_closed());
    }

    #[test]
    fn test_client_connection_handle() {
        let remote = "127.0.0.1:8080".parse().unwrap();
        let conn = ClientConnection::new(remote);
        let handle = ClientConnectionHandle::new(1, conn);

        assert_eq!(handle.id(), 1);
        assert!(handle.try_lock().is_ok());
    }

    #[test]
    fn test_connection_state_transitions() {
        let remote = "127.0.0.1:8080".parse().unwrap();
        let mut conn = ClientConnection::new(remote);

        assert_eq!(conn.state(), ConnectionState::Connecting);

        conn.set_connected();
        assert_eq!(conn.state(), ConnectionState::Connected);
        assert!(conn.is_connected());
    }

    #[tokio::test]
    async fn test_message_sending() {
        let remote = "127.0.0.1:8080".parse().unwrap();
        let mut conn = ClientConnection::new(remote);

        // Exercise the send APIs; errors are acceptable without a real transport stream
        let _ = conn.send_text("Hello").await;
        let _ = conn.send_binary(&[1u8, 2, 3][..]).await;
        let _ = conn.ping(None).await;
        let _ = conn.pong(None).await;
    }
}
