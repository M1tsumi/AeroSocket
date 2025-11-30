//! WebSocket server implementation
//!
//! This module provides the main server implementation for handling WebSocket connections.

use crate::{
    config::ServerConfig,
    handler::{BoxedHandler, Handler},
    connection::{Connection, ConnectionHandle},
    rate_limit::RateLimitMiddleware,
};
use aerosocket_core::{Error, Result, Transport};
use aerosocket_core::transport::TransportStream;
use aerosocket_core::handshake::{HandshakeConfig, parse_client_handshake, validate_client_handshake, create_server_handshake, response_to_string};
use aerosocket_core::error::ConfigError;
use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{timeout, Duration};

/// WebSocket server
pub struct Server {
    config: ServerConfig,
    handler: BoxedHandler,
    rate_limiter: Option<Arc<RateLimitMiddleware>>,
}

/// Connection manager for tracking active connections
#[derive(Debug)]
pub struct ConnectionManager {
    connections: Arc<Mutex<HashMap<u64, ConnectionHandle>>>,
    next_id: Arc<Mutex<u64>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Add a new connection
    pub async fn add_connection(&self, connection: Connection) -> u64 {
        let mut next_id = self.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;

        let handle = ConnectionHandle::new(id, connection);
        let mut connections = self.connections.lock().await;
        connections.insert(id, handle);
        id
    }

    /// Remove a connection
    pub async fn remove_connection(&self, id: u64) -> Option<ConnectionHandle> {
        let mut connections = self.connections.lock().await;
        connections.remove(&id)
    }

    /// Get a connection by ID
    pub async fn get_connection(&self, id: u64) -> Option<ConnectionHandle> {
        let connections = self.connections.lock().await;
        connections.get(&id).cloned()
    }

    /// Get all active connections
    pub async fn get_all_connections(&self) -> Vec<ConnectionHandle> {
        let connections = self.connections.lock().await;
        connections.values().cloned().collect()
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.lock().await;
        connections.len()
    }
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("config", &self.config)
            .field("handler", &"<handler>")
            .finish()
    }
}

impl Server {
    /// Create a new server with the given config and handler
    pub fn new(config: ServerConfig, handler: BoxedHandler) -> Self {
        let rate_limiter = if config.backpressure.enabled {
            Some(Arc::new(RateLimitMiddleware::new(crate::rate_limit::RateLimitConfig {
                max_requests: config.backpressure.max_requests_per_minute,
                window: Duration::from_secs(60),
                max_connections: config.max_connections / 10, // 10% of max connections per IP
                connection_timeout: config.idle_timeout,
            })))
        } else {
            None
        };

        Self { 
            config, 
            handler,
            rate_limiter,
        }
    }

    /// Create a server builder
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    /// Start serving connections
    pub async fn serve(self) -> Result<()> {
        let connection_manager = Arc::new(ConnectionManager::new());
        self.serve_with_connection_manager(connection_manager).await
    }

    /// Start serving with graceful shutdown
    pub async fn serve_with_graceful_shutdown<F>(self, shutdown_signal: F) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let connection_manager = Arc::new(ConnectionManager::new());
        let shutdown_signal = Box::pin(shutdown_signal);
        self.serve_with_connection_manager_and_shutdown(connection_manager, shutdown_signal).await
    }

    /// Internal serve method
    async fn serve_with_connection_manager(self, connection_manager: Arc<ConnectionManager>) -> Result<()> {
        let (_shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let shutdown_rx = Box::pin(async move {
            let _ = shutdown_rx.recv().await;
        });

        self.serve_with_connection_manager_and_shutdown(connection_manager, shutdown_rx).await
    }

    /// Internal serve method with shutdown signal
    async fn serve_with_connection_manager_and_shutdown<F>(
        self,
        _connection_manager: Arc<ConnectionManager>,
        _shutdown_signal: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + Unpin,
    {
        // Create transport based on configuration
        #[cfg(feature = "tcp-transport")]
        {
            if self.config.transport_type == crate::config::TransportType::Tcp {
                let transport = crate::tcp_transport::TcpTransport::bind(self.config.bind_address).await?;
                return self.serve_with_tcp_transport(transport, _connection_manager, _shutdown_signal).await;
            }
        }
        
        #[cfg(feature = "tls-transport")]
        {
            if self.config.transport_type == crate::config::TransportType::Tls {
                let _tls_config = self.config.tls.as_ref()
                    .ok_or_else(|| Error::Other("TLS configuration required for TLS transport".to_string()))?;
                // Note: TLS transport is disabled in this release
                return Err(Error::Other("TLS transport is not available in this release. Please enable the 'tls-transport' feature and implement proper TLS configuration.".to_string()));
            }
        }

        // Fallback to TCP if no specific transport is configured
        if self.config.transport_type == crate::config::TransportType::Tcp {
            #[cfg(feature = "tcp-transport")]
            {
                let transport = crate::tcp_transport::TcpTransport::bind(self.config.bind_address).await?;
                return self.serve_with_tcp_transport(transport, _connection_manager, _shutdown_signal).await;
            }
        }
        Err(Error::Config(ConfigError::Validation("No transport available".to_string())))
    }

    /// Serve with TCP transport
    #[cfg(feature = "tcp-transport")]
    async fn serve_with_tcp_transport<F>(
        self,
        transport: crate::tcp_transport::TcpTransport,
        connection_manager: Arc<ConnectionManager>,
        mut shutdown_signal: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + Unpin,
    {
        // Spawn connection handling task
        let handler = self.handler;
        let config = self.config.clone();
        let manager = connection_manager.clone();
        let rate_limiter = self.rate_limiter.clone();
        
        let server_task = tokio::spawn(async move {
            let mut connection_counter = 0u64;
            
            loop {
                // Check for shutdown
                tokio::select! {
                    result = transport.accept() => {
                        match result {
                            Ok(mut stream) => {
                                // Get remote address for rate limiting
                                let remote_addr = match stream.remote_addr() {
                                    Ok(addr) => addr.ip(),
                                    Err(e) => {
                                        crate::log_error!("Failed to get remote address: {:?}", e);
                                        let _ = stream.close().await;
                                        continue;
                                    }
                                };

                                // Check rate limiting if enabled
                                if let Some(ref rate_limiter) = rate_limiter {
                                    if !rate_limiter.check_connection(remote_addr).await.unwrap_or(true) {
                                        crate::log_warn!("Rate limit exceeded for IP: {}", remote_addr);
                                        let _ = stream.close().await;
                                        continue;
                                    }
                                }

                                // Check connection limit
                                if manager.connection_count().await >= config.max_connections {
                                    crate::log_warn!("Connection limit reached, rejecting connection from {}", remote_addr);
                                    // Close the stream gracefully
                                    let _ = stream.close().await;
                                    continue;
                                }

                                connection_counter += 1;
                                let manager = manager.clone();
                                let handler = handler.clone();
                                let config = config.clone();
                                let rate_limiter = rate_limiter.clone();
                                
                                // Spawn connection handler
                                tokio::spawn(async move {
                                    if let Err(e) = Self::handle_connection(
                                        stream,
                                        handler,
                                        config,
                                        manager,
                                        rate_limiter,
                                    ).await {
                                        crate::log_error!("Connection handling error: {:?}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                crate::log_error!("Accept error: {:?}", e);
                                // Continue accepting other connections
                            }
                        }
                    }
                    _ = &mut shutdown_signal => {
                        break;
                    }
                }
            }
        });

        // Wait for shutdown signal or server task completion
        tokio::select! {
            result = server_task => {
                match result {
                    Ok(()) => Ok(()),
                    Err(e) => Err(Error::Other(format!("Server task panicked: {}", e))),
                }
            }
            _ = &mut shutdown_signal => {
                // Graceful shutdown
                self.graceful_shutdown(connection_manager).await?;
                Ok(())
            }
        }
    }

    /// Serve with TLS transport
    #[cfg(feature = "tls-transport")]
    async fn serve_with_tls_transport<F>(
        self,
        _transport: crate::tls_transport::TlsTransport,
        _connection_manager: Arc<ConnectionManager>,
        _shutdown_signal: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + Unpin + 'static,
    {
        Err(Error::Other("TLS transport is not available in this release".to_string()))
    }

    /// Handle a single TLS connection
    #[cfg(feature = "tls-transport")]
    async fn handle_tls_connection(
        _stream: crate::tls_transport::TlsStreamWrapper,
        _handler: BoxedHandler,
        _config: ServerConfig,
        _connection_manager: Arc<ConnectionManager>,
        _rate_limiter: Option<Arc<RateLimitMiddleware>>,
    ) -> Result<()> {
        Err(Error::Other("TLS connection handling is not available in this release".to_string()))
    }

    /// Handle a single connection
    async fn handle_connection(
        mut stream: crate::tcp_transport::TcpStream,
        handler: BoxedHandler,
        config: ServerConfig,
        connection_manager: Arc<ConnectionManager>,
        rate_limiter: Option<Arc<RateLimitMiddleware>>,
    ) -> Result<()> {
        // Perform WebSocket handshake
        let (remote_addr, local_addr) = Self::perform_handshake(&mut stream, &config).await?;
        
        // Convert to boxed transport stream
        let boxed_stream: Box<dyn TransportStream> = Box::new(stream);
        
        // Create connection with stream
        let connection = Connection::with_stream(remote_addr, local_addr, boxed_stream);
        
        // Add to connection manager
        let connection_id = connection_manager.add_connection(connection).await;
        
        // Get connection handle
        let connection_handle = connection_manager.get_connection(connection_id).await
            .ok_or_else(|| Error::Other("Failed to get connection handle".to_string()))?;

        // Call handler
        if let Err(e) = handler.handle(connection_handle).await {
            crate::log_error!("Handler error: {:?}", e);
        }

        // Remove connection from manager
        connection_manager.remove_connection(connection_id).await;
        
        // Clean up rate limiting
        if let Some(ref rate_limiter) = rate_limiter {
            rate_limiter.connection_closed(remote_addr.ip()).await;
        }
        
        Ok(())
    }

    /// Perform WebSocket handshake over TLS
    #[cfg(feature = "tls-transport")]
    async fn perform_tls_handshake(
        stream: &mut crate::tls_transport::TlsStreamWrapper,
        config: &ServerConfig,
    ) -> Result<(SocketAddr, SocketAddr)> {
        // Read HTTP request over TLS
        let request_data = Self::read_tls_handshake_request(stream, config.handshake_timeout).await?;
        let request_str = String::from_utf8_lossy(&request_data);
        
        // Parse handshake request
        let request = parse_client_handshake(&request_str)?;
        
        // Create handshake config
        let handshake_config = HandshakeConfig {
            protocols: config.supported_protocols.clone(),
            extensions: config.supported_extensions.clone(),
            origin: config.allowed_origin.clone(),
            host: None,
            extra_headers: config.extra_headers.clone(),
        };
        
        // Validate request
        validate_client_handshake(&request, &handshake_config)?;
        
        // Create response
        let response = create_server_handshake(&request, &handshake_config)?;
        let response_str = response_to_string(&response);
        
        // Send response over TLS
        stream.write_all(response_str.as_bytes()).await?;
        stream.flush().await?;
        
        // Get addresses
        let remote_addr = stream.remote_addr()?;
        let local_addr = stream.local_addr()?;
        
        Ok((remote_addr, local_addr))
    }

    /// Read handshake request from TLS stream
    #[cfg(feature = "tls-transport")]
    async fn read_tls_handshake_request(
        stream: &mut crate::tls_transport::TlsStreamWrapper,
        timeout_duration: Duration,
    ) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut temp_buffer = [0u8; 1024];
        
        let read_result = timeout(timeout_duration, async {
            loop {
                let n = stream.read(&mut temp_buffer).await?;
                if n == 0 {
                    break;
                }
                
                buffer.extend_from_slice(&temp_buffer[..n]);
                
                // Check for end of headers (double CRLF)
                if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
                
                // Prevent reading too much
                if buffer.len() > 8192 {
                    return Err(Error::Other("TLS handshake request too large".to_string()));
                }
            }
            
            Ok::<(), Error>(())
        }).await;
        
        match read_result {
            Ok(result) => {
                result?;
                Ok(buffer)
            },
            Err(_) => Err(Error::Other("TLS handshake timeout".to_string())),
        }
    }

    /// Perform WebSocket handshake
    async fn perform_handshake(
        stream: &mut crate::tcp_transport::TcpStream,
        config: &ServerConfig,
    ) -> Result<(SocketAddr, SocketAddr)> {
        // Read HTTP request
        let request_data = Self::read_handshake_request(stream, config.handshake_timeout).await?;
        let request_str = String::from_utf8_lossy(&request_data);
        
        // Parse handshake request
        let request = parse_client_handshake(&request_str)?;
        
        // Create handshake config
        let handshake_config = HandshakeConfig {
            protocols: config.supported_protocols.clone(),
            extensions: config.supported_extensions.clone(),
            origin: config.allowed_origin.clone(),
            host: None,
            extra_headers: config.extra_headers.clone(),
        };
        
        // Validate request
        validate_client_handshake(&request, &handshake_config)?;
        
        // Create response
        let response = create_server_handshake(&request, &handshake_config)?;
        let response_str = response_to_string(&response);
        
        // Send response
        stream.write_all(response_str.as_bytes()).await?;
        stream.flush().await?;
        
        // Get addresses
        let remote_addr = stream.remote_addr()?;
        let local_addr = stream.local_addr()?;
        
        Ok((remote_addr, local_addr))
    }

    /// Read handshake request from stream
    async fn read_handshake_request(
        stream: &mut crate::tcp_transport::TcpStream,
        timeout_duration: Duration,
    ) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut temp_buffer = [0u8; 1024];
        
        let read_result = timeout(timeout_duration, async {
            loop {
                let n = stream.read(&mut temp_buffer).await?;
                if n == 0 {
                    break;
                }
                
                buffer.extend_from_slice(&temp_buffer[..n]);
                
                // Check for end of headers (double CRLF)
                if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
                
                // Prevent reading too much
                if buffer.len() > 8192 {
                    return Err(Error::Other("Handshake request too large".to_string()));
                }
            }
            
            Ok::<(), Error>(())
        }).await;
        
        match read_result {
            Ok(result) => {
                result?;
                Ok(buffer)
            },
            Err(_) => Err(Error::Other("Handshake timeout".to_string())),
        }
    }

    /// Perform graceful shutdown
    async fn graceful_shutdown(&self, connection_manager: Arc<ConnectionManager>) -> Result<()> {
        // Get all connections
        let connections = connection_manager.get_all_connections().await;
        
        // Send close frames to all connections
        for handle in connections {
            if let Ok(mut connection) = handle.try_lock().await {
                let _ = connection.close(Some(1000), Some("Server shutdown")).await;
            }
        }
        
        Ok(())
    }
}

/// Server builder
#[derive(Debug, Clone)]
pub struct ServerBuilder {
    config: ServerConfig,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
        }
    }

    /// Bind to the given address
    pub fn bind<A: std::net::ToSocketAddrs>(mut self, addr: A) -> Result<Self> {
        self.config.bind_address = addr.to_socket_addrs()?.next().ok_or_else(|| {
            Error::Config(ConfigError::Validation("Invalid bind address".to_string()))
        })?;
        Ok(self)
    }

    /// Set maximum connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
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

    /// Set idle timeout
    pub fn idle_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.config.idle_timeout = timeout;
        self
    }

    /// Enable/disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.config.compression.enabled = enabled;
        self
    }

    /// Set backpressure strategy
    pub fn backpressure(mut self, strategy: crate::config::BackpressureStrategy) -> Self {
        self.config.backpressure.strategy = strategy;
        self
    }

    /// Build the server
    pub fn build(self) -> Result<Server> {
        // Validate configuration
        self.config.validate()?;

        // Create default handler
        let handler = Box::new(crate::handler::DefaultHandler::new());

        Ok(Server::new(self.config, handler))
    }

    /// Build the server with a custom handler
    pub fn build_with_handler<H>(self, handler: H) -> Result<Server>
    where
        H: Handler + Send + Sync + 'static,
    {
        // Validate configuration
        self.config.validate()?;

        Ok(Server::new(self.config, Box::new(handler)))
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let builder = ServerBuilder::new()
            .bind("127.0.0.1:8080")
            .unwrap()
            .max_connections(1000)
            .max_frame_size(1024 * 1024)
            .compression(true);

        assert!(builder.build().is_ok());
    }
}
