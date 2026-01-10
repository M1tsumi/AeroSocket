//! WebSocket server implementation
//!
//! This module provides the main server implementation for handling WebSocket connections.

use crate::{
    config::ServerConfig,
    connection::{Connection, ConnectionHandle},
    handler::{BoxedHandler, Handler},
    rate_limit::RateLimitMiddleware,
};
use aerosocket_core::error::ConfigError;
use aerosocket_core::handshake::{
    create_server_handshake, parse_client_handshake, response_to_string, validate_client_handshake,
    HandshakeConfig,
};
use aerosocket_core::transport::TransportStream;
use aerosocket_core::{Error, Result, Transport};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};

/// WebSocket server
pub struct Server {
    config: ServerConfig,
    handler: BoxedHandler,
    rate_limiter: Option<Arc<RateLimitMiddleware>>,
    manager: Arc<ConnectionManager>,
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

    /// Broadcast binary message to all connections
    pub async fn broadcast_binary_to_all(&self, data: &[u8]) -> Result<()> {
        let data = data.to_vec();
        let connections = self.get_all_connections().await;
        for handle in connections {
            if let Ok(mut conn) = handle.try_lock().await {
                let _ = conn.send_binary(data.clone()).await;
            }
        }
        Ok(())
    }

    /// Broadcast text message to all connections
    pub async fn broadcast_text_to_all(&self, text: &str) -> Result<()> {
        let connections = self.get_all_connections().await;
        for handle in connections {
            if let Ok(mut conn) = handle.try_lock().await {
                let _ = conn.send_text(text).await;
            }
        }
        Ok(())
    }

    /// Broadcast binary message to all connections except the specified one
    pub async fn broadcast_binary_except(&self, data: &[u8], except_id: u64) -> Result<()> {
        let data = data.to_vec();
        let connections = self.get_all_connections().await;
        for handle in connections {
            if handle.id() != except_id {
                if let Ok(mut conn) = handle.try_lock().await {
                    let _ = conn.send_binary(data.clone()).await;
                }
            }
        }
        Ok(())
    }

    /// Broadcast text message to all connections except the specified one
    pub async fn broadcast_text_except(&self, text: &str, except_id: u64) -> Result<()> {
        let connections = self.get_all_connections().await;
        for handle in connections {
            if handle.id() != except_id {
                if let Ok(mut conn) = handle.try_lock().await {
                    let _ = conn.send_text(text).await;
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("config", &self.config)
            .field("handler", &"<handler>")
            .field("manager", &self.manager)
            .finish()
    }
}

impl Server {
    /// Create a new server with the given config and handler
    pub fn new(config: ServerConfig, handler: BoxedHandler) -> Self {
        let rate_limiter = if config.backpressure.enabled {
            Some(Arc::new(RateLimitMiddleware::new(
                crate::rate_limit::RateLimitConfig {
                    max_requests: config.backpressure.max_requests_per_minute,
                    window: Duration::from_secs(60),
                    max_connections: config.max_connections / 10, // 10% of max connections per IP
                    connection_timeout: config.idle_timeout,
                },
            )))
        } else {
            None
        };

        Self {
            config,
            handler,
            rate_limiter,
            manager: Arc::new(ConnectionManager::new()),
        }
    }

    /// Create a server builder
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    /// Start serving connections
    pub async fn serve(self) -> Result<()> {
        let manager = self.manager.clone();
        self.serve_with_connection_manager(manager).await
    }

    /// Start serving with graceful shutdown
    pub async fn serve_with_graceful_shutdown<F>(self, shutdown_signal: F) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let shutdown_signal = Box::pin(shutdown_signal);
        let manager = self.manager.clone();
        self.serve_with_connection_manager_and_shutdown(manager, shutdown_signal)
            .await
    }

    /// Internal serve method
    async fn serve_with_connection_manager(
        self,
        connection_manager: Arc<ConnectionManager>,
    ) -> Result<()> {
        let (_shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let shutdown_rx = Box::pin(async move {
            let _ = shutdown_rx.recv().await;
        });

        self.serve_with_connection_manager_and_shutdown(connection_manager, shutdown_rx)
            .await
    }

    /// Internal serve method with shutdown signal
    async fn serve_with_connection_manager_and_shutdown<F>(
        self,
        _connection_manager: Arc<ConnectionManager>,
        _shutdown_signal: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + Unpin + 'static,
    {
        // Create transport based on configuration
        #[cfg(feature = "tcp-transport")]
        {
            if self.config.transport_type == crate::config::TransportType::Tcp {
                let transport =
                    crate::tcp_transport::TcpTransport::bind(self.config.bind_address).await?;
                return self
                    .serve_with_tcp_transport(transport, _connection_manager, _shutdown_signal)
                    .await;
            }
        }

        #[cfg(feature = "tls-transport")]
        {
            if self.config.transport_type == crate::config::TransportType::Tls {
                let tls_config = self.config.tls.as_ref().ok_or_else(|| {
                    Error::Other("TLS configuration required for TLS transport".to_string())
                })?;

                let server_config = crate::config::build_rustls_server_config(tls_config)?;
                let transport = crate::tls_transport::TlsTransport::bind(
                    self.config.bind_address,
                    server_config,
                )
                .await?;

                return self
                    .serve_with_tls_transport(transport, _connection_manager, _shutdown_signal)
                    .await;
            }
        }

        // Fallback to TCP if no specific transport is configured
        if self.config.transport_type == crate::config::TransportType::Tcp {
            #[cfg(feature = "tcp-transport")]
            {
                let transport =
                    crate::tcp_transport::TcpTransport::bind(self.config.bind_address).await?;
                return self
                    .serve_with_tcp_transport(transport, _connection_manager, _shutdown_signal)
                    .await;
            }
        }
        Err(Error::Config(ConfigError::Validation(
            "No transport available".to_string(),
        )))
    }

    /// Serve with TCP transport
    #[cfg(feature = "tcp-transport")]
    async fn serve_with_tcp_transport<F>(
        self,
        transport: crate::tcp_transport::TcpTransport,
        connection_manager: Arc<ConnectionManager>,
        _shutdown_signal: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
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
                                crate::log_debug!("Accepted connection #{} from {}", connection_counter, remote_addr);
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
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                }
            }
        });

        // Wait for server task completion
        match server_task.await {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::Other(format!("Server task panicked: {}", e))),
        }
    }

    /// Serve with TLS transport
    #[cfg(feature = "tls-transport")]
    async fn serve_with_tls_transport<F>(
        self,
        transport: crate::tls_transport::TlsTransport,
        connection_manager: Arc<ConnectionManager>,
        _shutdown_signal: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + Unpin + 'static,
    {
        let handler = self.handler;
        let config = self.config.clone();
        let manager = connection_manager.clone();
        let rate_limiter = self.rate_limiter.clone();

        let server_task = tokio::spawn(async move {
            let mut connection_counter = 0u64;

            loop {
                tokio::select! {
                    result = transport.accept() => {
                        match result {
                            Ok(mut stream) => {
                                let remote_ip = match stream.remote_addr() {
                                    Ok(addr) => addr.ip(),
                                    Err(e) => {
                                        crate::log_error!("Failed to get remote address: {:?}", e);
                                        let _ = stream.close().await;
                                        continue;
                                    }
                                };

                                if let Some(ref rate_limiter) = rate_limiter {
                                    if !rate_limiter.check_connection(remote_ip).await.unwrap_or(true) {
                                        crate::log_warn!("Rate limit exceeded for IP: {}", remote_ip);
                                        let _ = stream.close().await;
                                        continue;
                                    }
                                }

                                if manager.connection_count().await >= config.max_connections {
                                    crate::log_warn!("Connection limit reached, rejecting TLS connection from {}", remote_ip);
                                    let _ = stream.close().await;
                                    continue;
                                }

                                connection_counter += 1;
                                crate::log_debug!(
                                    "Accepted TLS connection #{} from {}",
                                    connection_counter,
                                    remote_ip
                                );

                                let manager = manager.clone();
                                let handler = handler.clone();
                                let config = config.clone();
                                let rate_limiter = rate_limiter.clone();

                                tokio::spawn(async move {
                                    if let Err(e) = Self::handle_tls_connection(
                                        stream,
                                        handler,
                                        config,
                                        manager,
                                        rate_limiter,
                                    )
                                    .await
                                    {
                                        crate::log_error!("TLS connection handling error: {:?}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                crate::log_error!("TLS accept error: {:?}", e);
                            }
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                }
            }
        });

        match server_task.await {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::Other(format!("Server task panicked: {}", e))),
        }
    }

    /// Handle a single TLS connection
    #[cfg(feature = "tls-transport")]
    async fn handle_tls_connection(
        mut stream: crate::tls_transport::TlsStreamWrapper,
        handler: BoxedHandler,
        config: ServerConfig,
        connection_manager: Arc<ConnectionManager>,
        rate_limiter: Option<Arc<RateLimitMiddleware>>,
    ) -> Result<()> {
        let (remote_addr, local_addr, endpoint) =
            Self::perform_tls_handshake(&mut stream, &config).await?;

        let boxed_stream: Box<dyn TransportStream> = Box::new(stream);
        let connection = Connection::with_stream(remote_addr, local_addr, boxed_stream);

        let connection_id = connection_manager.add_connection(connection).await;

        #[cfg(feature = "prometheus")]
        {
            let active = connection_manager.connection_count().await as f64;
            metrics::gauge!("aerosocket_server_active_connections").set(active);
            metrics::counter!("aerosocket_server_connections_opened_total").increment(1);
            metrics::counter!("aerosocket_server_endpoint_connections_opened_total").increment(1);
        }

        let connection_handle = connection_manager
            .get_connection(connection_id)
            .await
            .ok_or_else(|| Error::Other("Failed to get connection handle".to_string()))?;

        if let Err(e) = handler.handle(connection_handle).await {
            crate::log_error!("Handler error: {:?}", e);
        }

        connection_manager.remove_connection(connection_id).await;

        if let Some(ref rate_limiter) = rate_limiter {
            rate_limiter.connection_closed(remote_addr.ip()).await;
        }

        #[cfg(feature = "prometheus")]
        {
            let active = connection_manager.connection_count().await as f64;
            metrics::gauge!("aerosocket_server_active_connections").set(active);
            metrics::counter!("aerosocket_server_connections_closed_total").increment(1);
        }

        Ok(())
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
        let (remote_addr, local_addr, endpoint, negotiated_extensions) =
            Self::perform_handshake(&mut stream, &config).await?;

        // Convert to boxed transport stream
        let boxed_stream: Box<dyn TransportStream> = Box::new(stream);

        // Create connection with stream
        let mut connection = Connection::with_stream(remote_addr, local_addr, boxed_stream);
        connection.metadata.compression_negotiated = negotiated_extensions.iter().any(|e| e.contains("permessage-deflate"));
        connection.metadata.extensions = negotiated_extensions;

        // Add to connection manager
        let connection_id = connection_manager.add_connection(connection).await;

        #[cfg(feature = "prometheus")]
        {
            let active = connection_manager.connection_count().await as f64;
            metrics::gauge!("aerosocket_server_active_connections").set(active);
            metrics::counter!("aerosocket_server_connections_opened_total").increment(1);
            metrics::counter!("aerosocket_server_endpoint_connections_opened_total").increment(1);
        }

        // Get connection handle
        let connection_handle = connection_manager
            .get_connection(connection_id)
            .await
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

        #[cfg(feature = "prometheus")]
        {
            let active = connection_manager.connection_count().await as f64;
            metrics::gauge!("aerosocket_server_active_connections").set(active);
            metrics::counter!("aerosocket_server_connections_closed_total").increment(1);
        }

        Ok(())
    }

    /// Perform WebSocket handshake over TLS
    #[cfg(feature = "tls-transport")]
    #[cfg_attr(feature = "logging", tracing::instrument(skip(stream, config)))]
    async fn perform_tls_handshake(
        stream: &mut crate::tls_transport::TlsStreamWrapper,
        config: &ServerConfig,
    ) -> Result<(SocketAddr, SocketAddr, String)> {
        let start = Instant::now();
        // Read HTTP request over TLS
        let request_data =
            Self::read_tls_handshake_request(stream, config.handshake_timeout).await?;
        let request_str = String::from_utf8_lossy(&request_data);

        // Parse handshake request
        let request = parse_client_handshake(&request_str)?;

        // Create handshake config
        let handshake_config = HandshakeConfig {
            protocols: config.supported_protocols.clone(),
            extensions: config.supported_extensions.clone(),
            origin: None,
            allowed_origins: config.allowed_origins.clone(),
            host: None,
            auth: None,
            compression: aerosocket_core::handshake::CompressionConfig {
                enabled: config.compression.enabled,
                client_max_window_bits: config.compression.client_max_window_bits,
                server_max_window_bits: config.compression.server_max_window_bits,
                compression_level: Some(config.compression.level as u32),
            },
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

        #[cfg(feature = "prometheus")]
        {
            let elapsed = start.elapsed().as_secs_f64();
            metrics::histogram!("aerosocket_server_handshake_duration_seconds").record(elapsed);
        }

        // Get addresses
        let remote_addr = stream.remote_addr()?;
        let local_addr = stream.local_addr()?;
        let endpoint = request.uri.clone();

        // Extract negotiated extensions from response headers
        let negotiated_extensions = if let Some(ext_header) = response.headers.get("Sec-WebSocket-Extensions") {
            ext_header.split(',').map(|s| s.trim().split(';').next().unwrap_or(s.trim()).to_string()).collect()
        } else {
            vec![]
        };

        Ok((remote_addr, local_addr, endpoint, negotiated_extensions))
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
        })
        .await;

        match read_result {
            Ok(result) => {
                result?;
                Ok(buffer)
            }
            Err(_) => Err(Error::Other("TLS handshake timeout".to_string())),
        }
    }

    /// Perform WebSocket handshake
    #[cfg_attr(feature = "logging", tracing::instrument(skip(stream, config)))]
    async fn perform_handshake(
        stream: &mut crate::tcp_transport::TcpStream,
        config: &ServerConfig,
    ) -> Result<(SocketAddr, SocketAddr, String, Vec<String>)> {
        let start = Instant::now();
        // Read HTTP request
        let request_data = Self::read_handshake_request(stream, config.handshake_timeout).await?;
        let request_str = String::from_utf8_lossy(&request_data);

        // Check if it's a WebSocket upgrade request
        if !request_str.contains("Upgrade: websocket") {
            // Handle as HTTP request
            return Self::handle_http_request(stream, &request_str, config).await;
        }

        // Parse handshake request
        let request = parse_client_handshake(&request_str)?;

        // Create handshake config
        let handshake_config = HandshakeConfig {
            protocols: config.supported_protocols.clone(),
            extensions: config.supported_extensions.clone(),
            origin: None,
            allowed_origins: config.allowed_origins.clone(),
            host: None,
            auth: None,
            compression: aerosocket_core::handshake::CompressionConfig {
                enabled: config.compression.enabled,
                client_max_window_bits: config.compression.client_max_window_bits,
                server_max_window_bits: config.compression.server_max_window_bits,
                compression_level: Some(config.compression.level as u32),
            },
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

        #[cfg(feature = "prometheus")]
        {
            let elapsed = start.elapsed().as_secs_f64();
            metrics::histogram!("aerosocket_server_handshake_duration_seconds").record(elapsed);
        }

        // Get addresses
        let remote_addr = stream.remote_addr()?;
        let local_addr = stream.local_addr()?;
        let endpoint = request.uri.clone();

        // Extract negotiated extensions from response headers
        let negotiated_extensions = if let Some(ext_header) = response.headers.get("Sec-WebSocket-Extensions") {
            ext_header.split(',').map(|s| s.trim().split(';').next().unwrap_or(s.trim()).to_string()).collect()
        } else {
            vec![]
        };

        Ok((remote_addr, local_addr, endpoint, negotiated_extensions))
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
        })
        .await;

        match read_result {
            Ok(result) => {
                result?;
                Ok(buffer)
            }
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

    /// Handle HTTP request (for /health, /metrics)
    async fn handle_http_request(
        stream: &mut crate::tcp_transport::TcpStream,
        request_str: &str,
        config: &ServerConfig,
    ) -> Result<(SocketAddr, SocketAddr, String, Vec<String>)> {
        // Parse basic HTTP request
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        req.parse(request_str.as_bytes()).map_err(|e| Error::Other(format!("HTTP parse error: {}", e)))?;

        let method = req.method.unwrap_or("GET");
        let path = req.path.unwrap_or("/");

        let response = match (method, path) {
            ("GET", "/health") => Self::handle_health_request(config),
            #[cfg(feature = "prometheus")]
            ("GET", "/metrics") => Self::handle_metrics_request(config).await,
            _ => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string(),
        };

        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;

        // Return dummy values since this is not a WebSocket connection
        let remote_addr = stream.remote_addr()?;
        let local_addr = stream.local_addr()?;
        Ok((remote_addr, local_addr, path.to_string(), vec![]))
    }

    /// Handle /health request
    fn handle_health_request(config: &ServerConfig) -> String {
        // Simple health check - can be extended with custom checks
        let status = "ok";
        let body = format!("{{\"status\":\"{}\"}}", status);
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        )
    }

    /// Handle /metrics request
    #[cfg(feature = "prometheus")]
    async fn handle_metrics_request(config: &ServerConfig) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let body = String::from_utf8(buffer).unwrap();
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        )
    }

    /// Broadcast binary message to all connections
    pub async fn broadcast_binary_to_all(&self, data: &[u8]) -> Result<()> {
        self.manager.broadcast_binary_to_all(data).await
    }

    /// Broadcast text message to all connections
    pub async fn broadcast_text_to_all(&self, text: &str) -> Result<()> {
        self.manager.broadcast_text_to_all(text).await
    }

    /// Broadcast binary message to all connections except the specified one
    pub async fn broadcast_binary_except(&self, data: &[u8], except_id: u64) -> Result<()> {
        self.manager.broadcast_binary_except(data, except_id).await
    }

    /// Broadcast text message to all connections except the specified one
    pub async fn broadcast_text_except(&self, text: &str, except_id: u64) -> Result<()> {
        self.manager.broadcast_text_except(text, except_id).await
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

    /// Configure TLS using certificate and key files (requires `tls-transport` feature)
    #[cfg(feature = "tls-transport")]
    pub fn tls(mut self, cert_file: impl Into<String>, key_file: impl Into<String>) -> Self {
        self.config.tls = Some(crate::config::TlsConfig::new(
            cert_file.into(),
            key_file.into(),
        ));
        self
    }

    /// Use TLS transport instead of TCP (requires `tls-transport` feature)
    #[cfg(feature = "tls-transport")]
    pub fn transport_tls(mut self) -> Self {
        self.config.transport_type = crate::config::TransportType::Tls;
        self
    }

    /// Add an allowed origin for CORS (empty list means allow all)
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.config.allowed_origins.push(origin.into());
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

    /// Build the server with a WASM-based handler loaded from a .wasm file
    #[cfg(feature = "wasm-handlers")]
    pub fn build_with_wasm_handler_from_file(
        self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Server> {
        let handler = crate::handler::WasmHandler::from_file(path.as_ref())?;
        self.build_with_handler(handler)
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
