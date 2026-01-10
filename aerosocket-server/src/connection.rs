//! WebSocket connection handling
//!
//! This module provides connection management for WebSocket clients.

use aerosocket_core::frame::Frame;
use aerosocket_core::protocol::Opcode;
use aerosocket_core::{transport::TransportStream, Message, Result};
use bytes::{Bytes, BytesMut};
use std::net::SocketAddr;
use std::time::Duration;

/// Represents a WebSocket connection
pub struct Connection {
    /// Remote address
    remote_addr: SocketAddr,
    /// Local address
    local_addr: SocketAddr,
    /// Connection state
    state: ConnectionState,
    /// Connection metadata
    pub metadata: ConnectionMetadata,
    /// Transport stream
    stream: Option<Box<dyn TransportStream>>,
    /// Idle timeout duration
    idle_timeout: Option<Duration>,
    /// Last activity timestamp
    last_activity: std::time::Instant,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("remote_addr", &self.remote_addr)
            .field("local_addr", &self.local_addr)
            .field("state", &self.state)
            .field("metadata", &self.metadata)
            .field("stream", &"<stream>")
            .finish()
    }
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
    /// Whether compression was negotiated
    pub compression_negotiated: bool,
}

impl Connection {
    /// Create a new connection
    pub fn new(remote_addr: SocketAddr, local_addr: SocketAddr) -> Self {
        let now = std::time::Instant::now();
        Self {
            remote_addr,
            local_addr,
            state: ConnectionState::Connecting,
            metadata: ConnectionMetadata {
                subprotocol: None,
                extensions: Vec::new(),
                established_at: now,
                last_activity_at: now,
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                compression_negotiated: false,
            },
            stream: None,
            idle_timeout: None,
            last_activity: now,
        }
    }

    /// Create a new connection with a transport stream
    pub fn with_stream(
        remote_addr: SocketAddr,
        local_addr: SocketAddr,
        stream: Box<dyn TransportStream>,
    ) -> Self {
        let now = std::time::Instant::now();
        Self {
            remote_addr,
            local_addr,
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
                compression_negotiated: false,
            },
            stream: Some(stream),
            idle_timeout: None,
            last_activity: now,
        }
    }

    /// Create a new connection with timeout settings
    pub fn with_timeout(
        remote_addr: SocketAddr,
        local_addr: SocketAddr,
        stream: Box<dyn TransportStream>,
        idle_timeout: Option<Duration>,
    ) -> Self {
        let now = std::time::Instant::now();
        Self {
            remote_addr,
            local_addr,
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
                compression_negotiated: false,
            },
            stream: Some(stream),
            idle_timeout,
            last_activity: now,
        }
    }

    /// Set the transport stream
    pub fn set_stream(&mut self, stream: Box<dyn TransportStream>) {
        self.stream = Some(stream);
        self.state = ConnectionState::Connected;
    }

    /// Get the remote address
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Get the local address
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Get the connection state
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Get the connection metadata
    pub fn metadata(&self) -> &ConnectionMetadata {
        &self.metadata
    }

    /// Check if the connection has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.idle_timeout {
            self.last_activity.elapsed() > timeout
        } else {
            false
        }
    }

    /// Get the time until the connection times out
    pub fn time_until_timeout(&self) -> Option<Duration> {
        self.idle_timeout.map(|timeout| {
            let elapsed = self.last_activity.elapsed();
            if elapsed >= timeout {
                Duration::ZERO
            } else {
                timeout - elapsed
            }
        })
    }

    /// Update the last activity timestamp
    fn update_activity(&mut self) {
        self.last_activity = std::time::Instant::now();
        self.metadata.last_activity_at = self.last_activity;
    }

    /// Set the idle timeout
    pub fn set_idle_timeout(&mut self, timeout: Option<Duration>) {
        self.idle_timeout = timeout;
    }

    /// Send a message
    pub async fn send(&mut self, message: Message) -> Result<()> {
        // Update activity timestamp before borrowing stream
        self.update_activity();

        if let Some(stream) = &mut self.stream {
            // Convert message to WebSocket frame
            let frame = match message {
                Message::Text(text) => Frame::text(text.as_bytes().to_vec()),
                Message::Binary(data) => Frame::binary(data.as_bytes().to_vec()),
                Message::Ping(data) => Frame::ping(data.as_bytes().to_vec()),
                Message::Pong(data) => Frame::pong(data.as_bytes().to_vec()),
                Message::Close(code_and_reason) => {
                    Frame::close(code_and_reason.code(), Some(code_and_reason.reason()))
                }
            };

            // Serialize frame to bytes
            let frame_bytes = frame.to_bytes();

            #[cfg(feature = "metrics")]
            {
                metrics::counter!("aerosocket_server_messages_sent_total").increment(1);
                metrics::counter!("aerosocket_server_bytes_sent_total")
                    .increment(frame_bytes.len() as u64);
                metrics::histogram!("aerosocket_server_frame_size_bytes")
                    .record(frame_bytes.len() as f64);
            }

            // Send frame
            stream.write_all(&frame_bytes).await?;
            stream.flush().await?;

            // Update metadata
            self.metadata.messages_sent += 1;
            self.metadata.bytes_sent += frame_bytes.len() as u64;

            Ok(())
        } else {
            Err(aerosocket_core::Error::Other(
                "Connection not established".to_string(),
            ))
        }
    }

    /// Send a text message
    pub async fn send_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        self.send(Message::text(text.as_ref().to_string())).await
    }

    /// Send a binary message
    pub async fn send_binary(&mut self, data: impl Into<Bytes>) -> Result<()> {
        self.send(Message::binary(data)).await
    }

    /// Send a ping message
    pub async fn ping(&mut self, data: Option<&[u8]>) -> Result<()> {
        self.send(Message::ping(data.map(|d| d.to_vec()))).await
    }

    /// Send a pong message
    pub async fn pong(&mut self, data: Option<&[u8]>) -> Result<()> {
        self.send(Message::pong(data.map(|d| d.to_vec()))).await
    }

    /// Send a pong message (convenience method)
    pub async fn send_pong(&mut self) -> Result<()> {
        self.pong(None).await
    }

    /// Receive the next message
    pub async fn next(&mut self) -> Result<Option<Message>> {
        // Update activity timestamp before borrowing stream
        self.update_activity();

        if let Some(stream) = &mut self.stream {
            let mut message_buffer = Vec::new();
            let mut final_frame = false;
            let mut opcode = None;

            // Keep reading frames until we get a complete message
            while !final_frame {
                // Read frame data
                let mut frame_buffer = BytesMut::new();

                // Read at least the frame header (2 bytes)
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

                // Parse the frame to determine how much more data we need
                match Frame::parse(&mut frame_buffer, self.metadata.compression_negotiated) {
                    Ok(frame) => {
                        // Handle control frames immediately
                        match frame.opcode {
                            Opcode::Ping => {
                                let ping_data = frame.payload.to_vec();
                                // Send pong response
                                stream.write_all(&Frame::pong(ping_data).to_bytes()).await?;
                                stream.flush().await?;
                                continue;
                            }
                            Opcode::Pong => {
                                // Handle pong response (update activity)
                                // Note: We can't call update_activity here due to borrowing,
                                // but activity is already updated at the start of next()
                                continue;
                            }
                            Opcode::Close => {
                                // Parse close frame
                                let close_code = if frame.payload.len() >= 2 {
                                    let code_bytes = &frame.payload[..2];
                                    u16::from_be_bytes([code_bytes[0], code_bytes[1]])
                                } else {
                                    1000 // Normal closure
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
                                // Handle data frames
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
                    Err(_e) => {
                        // Need more data - read from stream
                        let mut temp_buf = [0u8; 1024];
                        match stream.read(&mut temp_buf).await {
                            Ok(0) => {
                                self.state = ConnectionState::Closed;
                                return Ok(None);
                            }
                            Ok(n) => {
                                frame_buffer.extend_from_slice(&temp_buf[..n]);
                            }
                            Err(e) => return Err(e),
                        }
                        continue;
                    }
                }
            }

            // Convert the collected message based on opcode
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
                    ))
                }
            };

            // Update metadata
            self.metadata.messages_received += 1;
            self.metadata.bytes_received += message_buffer.len() as u64;

            #[cfg(feature = "metrics")]
            {
                metrics::counter!("aerosocket_server_messages_received_total").increment(1);
                metrics::counter!("aerosocket_server_bytes_received_total")
                    .increment(message_buffer.len() as u64);
                metrics::histogram!("aerosocket_server_message_size_bytes")
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
    pub async fn close(&mut self, code: Option<u16>, reason: Option<&str>) -> Result<()> {
        self.state = ConnectionState::Closing;
        self.send(Message::close(code, reason.map(|s| s.to_string())))
            .await
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
}

/// Connection handle for managing connections
#[derive(Debug, Clone)]
pub struct ConnectionHandle {
    /// Connection ID
    id: u64,
    /// Connection reference
    connection: std::sync::Arc<tokio::sync::Mutex<Connection>>,
}

impl ConnectionHandle {
    /// Create a new connection handle
    pub fn new(id: u64, connection: Connection) -> Self {
        Self {
            id,
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        }
    }

    /// Get the connection ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Try to lock the connection
    pub async fn try_lock(&self) -> Result<tokio::sync::MutexGuard<'_, Connection>> {
        self.connection
            .try_lock()
            .map_err(|_| aerosocket_core::Error::Other("Failed to lock connection".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_creation() {
        let remote = "127.0.0.1:12345".parse().unwrap();
        let local = "127.0.0.1:8080".parse().unwrap();
        let conn = Connection::new(remote, local);

        assert_eq!(conn.remote_addr(), remote);
        assert_eq!(conn.local_addr(), local);
        assert_eq!(conn.state(), ConnectionState::Connecting);
        assert!(!conn.is_connected());
        assert!(!conn.is_closed());
    }

    #[tokio::test]
    async fn test_connection_handle() {
        let remote = "127.0.0.1:12345".parse().unwrap();
        let local = "127.0.0.1:8080".parse().unwrap();
        let conn = Connection::new(remote, local);
        let handle = ConnectionHandle::new(1, conn);

        assert_eq!(handle.id(), 1);
        assert!(handle.try_lock().await.is_ok());
    }
}
