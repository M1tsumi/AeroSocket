//! Transport layer abstraction
//!
//! This module provides a transport abstraction that allows AeroSocket to work
//! with different underlying transports (TCP, TLS, QUIC, etc.).

use crate::error::Result;

/// Transport trait for abstracting different transport types
#[async_trait::async_trait]
pub trait Transport: Send + Sync + 'static {
    /// The stream type produced by this transport
    type Stream: TransportStream;

    /// Accept an incoming connection
    async fn accept(&self) -> Result<Self::Stream>;

    /// Get the local address
    fn local_addr(&self) -> Result<std::net::SocketAddr>;

    /// Close the transport
    async fn close(self) -> Result<()>;
}

/// Trait for transport streams
#[async_trait::async_trait]
pub trait TransportStream: Send + Sync {
    /// Read data from the stream
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Write data to the stream
    async fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Write all data to the stream
    async fn write_all(&mut self, buf: &[u8]) -> Result<()>;

    /// Flush the stream
    async fn flush(&mut self) -> Result<()>;

    /// Close the stream
    async fn close(&mut self) -> Result<()>;

    /// Get the remote address
    fn remote_addr(&self) -> Result<std::net::SocketAddr>;

    /// Get the local address
    fn local_addr(&self) -> Result<std::net::SocketAddr>;
}

/// Configuration for transport options
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Maximum frame size
    pub max_frame_size: usize,
    /// Maximum message size
    pub max_message_size: usize,
    /// Handshake timeout
    pub handshake_timeout: std::time::Duration,
    /// Idle timeout
    pub idle_timeout: std::time::Duration,
    /// Enable Nagle's algorithm
    pub nodelay: bool,
    /// Receive buffer size
    pub recv_buffer_size: Option<usize>,
    /// Send buffer size
    pub send_buffer_size: Option<usize>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            max_frame_size: crate::protocol::constants::DEFAULT_MAX_FRAME_SIZE,
            max_message_size: crate::protocol::constants::DEFAULT_MAX_MESSAGE_SIZE,
            handshake_timeout: crate::protocol::constants::DEFAULT_HANDSHAKE_TIMEOUT,
            idle_timeout: crate::protocol::constants::DEFAULT_IDLE_TIMEOUT,
            nodelay: true,
            recv_buffer_size: None,
            send_buffer_size: None,
        }
    }
}

/// TCP transport implementation
#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
pub mod tcp {
    use super::*;

    /// TCP transport
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct TcpTransport {
        config: TransportConfig,
    }

    impl TcpTransport {
        /// Create a new TCP transport bound to the given address
        pub async fn bind(_addr: std::net::SocketAddr, config: TransportConfig) -> Result<Self> {
            // Placeholder implementation
            Ok(Self { config })
        }

        /// Create a new TCP transport with default config
        pub async fn bind_default(addr: std::net::SocketAddr) -> Result<Self> {
            Self::bind(addr, TransportConfig::default()).await
        }
    }

    /// TCP stream wrapper
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct TcpStream {
        config: TransportConfig,
    }

    impl TcpStream {
        /// Create a new TCP stream
        pub fn new(config: TransportConfig) -> Self {
            Self { config }
        }
    }
}

/// TLS transport implementation
#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
#[cfg(feature = "transport-tls")]
pub mod tls {
    use super::*;

    /// TLS transport
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct TlsTransport {
        config: TransportConfig,
    }

    impl TlsTransport {
        /// Create a new TLS transport bound to the given address
        pub async fn bind(_addr: std::net::SocketAddr, config: TransportConfig) -> Result<Self> {
            // Placeholder implementation
            Ok(Self { config })
        }
    }

    /// TLS stream wrapper
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct TlsStream {
        config: TransportConfig,
    }

    impl TlsStream {
        /// Create a new TLS stream
        pub fn new(config: TransportConfig) -> Self {
            Self { config }
        }
    }
}

/// Mock transport for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::mpsc;

    /// Mock transport for testing
    #[derive(Debug)]
    pub struct MockTransport {
        receiver: mpsc::Receiver<MockStream>,
        local_addr: std::net::SocketAddr,
    }

    impl MockTransport {
        /// Create a new mock transport
        pub fn new() -> (Self, mpsc::Sender<MockStream>) {
            let (sender, receiver) = mpsc::channel();
            let transport = Self {
                receiver,
                local_addr: "127.0.0.1:0".parse().unwrap(),
            };
            (transport, sender)
        }

        /// Accept a connection (blocking for testing)
        pub fn accept(&self) -> Result<MockStream> {
            match self.receiver.recv() {
                Ok(stream) => Ok(stream),
                Err(_) => Err(crate::Error::Connection(
                    "Mock transport closed".to_string(),
                )),
            }
        }

        /// Get local address
        pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
            Ok(self.local_addr)
        }
    }

    /// Mock stream for testing
    #[derive(Debug, Clone)]
    pub struct MockStream {
        data: Vec<u8>,
        remote_addr: std::net::SocketAddr,
        local_addr: std::net::SocketAddr,
    }

    impl MockStream {
        /// Create a new mock stream
        pub fn new(data: Vec<u8>) -> Self {
            Self {
                data,
                remote_addr: "127.0.0.1:12345".parse().unwrap(),
                local_addr: "127.0.0.1:8080".parse().unwrap(),
            }
        }

        /// Read data (blocking for testing)
        pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let to_copy = std::cmp::min(buf.len(), self.data.len());
            buf[..to_copy].copy_from_slice(&self.data[..to_copy]);
            self.data.drain(0..to_copy);
            Ok(to_copy)
        }

        /// Write data (blocking for testing)
        pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
            self.data.extend_from_slice(buf);
            Ok(buf.len())
        }

        /// Get remote address
        pub fn remote_addr(&self) -> Result<std::net::SocketAddr> {
            Ok(self.remote_addr)
        }

        /// Get local address
        pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
            Ok(self.local_addr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(
            config.max_frame_size,
            crate::protocol::constants::DEFAULT_MAX_FRAME_SIZE
        );
        assert!(config.nodelay);
    }

    #[cfg(test)]
    mod mock_tests {
        use crate::transport::mock::*;

        #[test]
        fn test_mock_transport() {
            let (transport, sender) = MockTransport::new();
            let stream = MockStream::new(b"hello".to_vec());
            sender.send(stream).unwrap();

            let accepted = transport.accept().unwrap();
            assert_eq!(
                accepted.remote_addr().unwrap().to_string(),
                "127.0.0.1:12345"
            );
        }

        #[test]
        fn test_mock_stream() {
            let mut stream = MockStream::new(Vec::new());

            let written = stream.write(b"hello").unwrap();
            assert_eq!(written, 5);

            let mut buf = [0u8; 10];
            let read = stream.read(&mut buf).unwrap();
            assert_eq!(read, 5);
            assert_eq!(&buf[..5], b"hello");
        }
    }
}
