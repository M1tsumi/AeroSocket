//! TCP transport implementation for AeroSocket
//!
//! This module provides TCP-based transport implementation for WebSocket connections.

use aerosocket_core::{Result, transport::{Transport, TransportStream}};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream as TokioTcpStream};

/// TCP transport implementation
#[derive(Debug)]
pub struct TcpTransport {
    listener: Option<TcpListener>,
    local_addr: SocketAddr,
}

impl TcpTransport {
    /// Create a new TCP transport bound to the given address
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(aerosocket_core::Error::Io)?;
        
        let local_addr = listener.local_addr()
            .map_err(aerosocket_core::Error::Io)?;
        
        Ok(Self {
            listener: Some(listener),
            local_addr,
        })
    }

    /// Create a new TCP transport without binding (for client connections)
    pub fn new_unbound() -> Self {
        Self {
            listener: None,
            local_addr: "0.0.0.0:0".parse().unwrap(),
        }
    }
}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::new_unbound()
    }
}

#[async_trait]
impl Transport for TcpTransport {
    type Stream = TcpStream;

    async fn accept(&self) -> Result<Self::Stream> {
        match &self.listener {
            Some(listener) => {
                let (stream, _addr) = listener.accept()
                    .await
                    .map_err(aerosocket_core::Error::Io)?;
                Ok(TcpStream::from_tokio(stream))
            }
            None => Err(aerosocket_core::Error::Other("Transport not bound".to_string())),
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

/// TCP stream implementation
#[derive(Debug)]
pub struct TcpStream {
    stream: Option<TokioTcpStream>,
    remote_addr: SocketAddr,
}

impl TcpStream {
    /// Create a new TCP stream from a tokio TCP stream
    pub fn from_tokio(stream: TokioTcpStream) -> Self {
        let remote_addr = stream.peer_addr()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
        
        Self {
            stream: Some(stream),
            remote_addr,
        }
    }

    /// Create a new TCP stream (unconnected)
    pub fn new() -> Self {
        Self {
            stream: None,
            remote_addr: "0.0.0.0:0".parse().unwrap(),
        }
    }

    /// Connect to a remote address
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(aerosocket_core::Error::Io)?;
        
        Ok(Self::from_tokio(stream))
    }
}

impl Default for TcpStream {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TransportStream for TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match &mut self.stream {
            Some(stream) => {
                use tokio::io::AsyncReadExt;
                let n = stream.read(buf)
                    .await
                    .map_err(aerosocket_core::Error::Io)?;
                Ok(n)
            }
            None => Err(aerosocket_core::Error::Other("Stream not connected".to_string())),
        }
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match &mut self.stream {
            Some(stream) => {
                use tokio::io::AsyncWriteExt;
                let n = stream.write(buf)
                    .await
                    .map_err(aerosocket_core::Error::Io)?;
                Ok(n)
            }
            None => Err(aerosocket_core::Error::Other("Stream not connected".to_string())),
        }
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match &mut self.stream {
            Some(stream) => {
                use tokio::io::AsyncWriteExt;
                stream.write_all(buf)
                    .await
                    .map_err(aerosocket_core::Error::Io)?;
                Ok(())
            }
            None => Err(aerosocket_core::Error::Other("Stream not connected".to_string())),
        }
    }

    async fn flush(&mut self) -> Result<()> {
        match &mut self.stream {
            Some(stream) => {
                use tokio::io::AsyncWriteExt;
                stream.flush()
                    .await
                    .map_err(aerosocket_core::Error::Io)?;
                Ok(())
            }
            None => Err(aerosocket_core::Error::Other("Stream not connected".to_string())),
        }
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            use tokio::io::AsyncWriteExt;
            stream.shutdown()
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
            Some(stream) => {
                stream.local_addr()
                    .map_err(aerosocket_core::Error::Io)
            }
            None => Err(aerosocket_core::Error::Other("Stream not connected".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_transport_creation() {
        let transport = TcpTransport::new();
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_tcp_stream_creation() {
        let stream = TcpStream::new();
        assert!(true); // Basic creation test
    }
}
