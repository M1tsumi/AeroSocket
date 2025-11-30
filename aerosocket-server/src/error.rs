//! Error handling and logging for the WebSocket server
//!
//! This module provides comprehensive error handling and logging capabilities
//! for the WebSocket server implementation.

use aerosocket_core::error::Error;
use std::fmt;
use std::io;

/// Server-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Handshake error
    #[error("Handshake error: {0}")]
    Handshake(#[from] HandshakeError),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    /// Transport error
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    /// Handler error
    #[error("Handler error: {0}")]
    Handler(#[from] HandlerError),

    /// Manager error
    #[error("Manager error: {0}")]
    Manager(#[from] ManagerError),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Core error
    #[error("Core error: {0}")]
    Core(#[from] Error),

    /// Timeout error
    #[error("Operation timed out after {duration:?}")]
    Timeout { duration: std::time::Duration },

    /// Capacity error
    #[error("Capacity exceeded: {0}")]
    Capacity(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Access denied: {0}")]
    Authorization(String),

    /// Rate limit error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Internal error
    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Invalid bind address
    #[error("Invalid bind address: {0}")]
    InvalidBindAddress(String),

    /// Invalid timeout value
    #[error("Invalid timeout value: {0}")]
    InvalidTimeout(String),

    /// Invalid buffer size
    #[error("Invalid buffer size: {0}")]
    InvalidBufferSize(String),

    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    /// Invalid TLS configuration
    #[error("Invalid TLS configuration: {0}")]
    InvalidTlsConfig(String),

    /// Invalid compression configuration
    #[error("Invalid compression configuration: {0}")]
    InvalidCompressionConfig(String),
}

/// Connection errors
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    /// Connection closed
    #[error("Connection closed")]
    Closed,

    /// Connection timed out
    #[error("Connection timed out")]
    TimedOut,

    /// Connection reset
    #[error("Connection reset by peer")]
    Reset,

    /// Connection refused
    #[error("Connection refused")]
    Refused,

    /// Connection limit exceeded
    #[error("Connection limit exceeded: {current}/{max}")]
    LimitExceeded { current: usize, max: usize },

    /// Invalid connection state
    #[error("Invalid connection state: {state}")]
    InvalidState { state: String },

    /// Connection not found
    #[error("Connection not found: {id}")]
    NotFound { id: u64 },

    /// Connection already exists
    #[error("Connection already exists: {id}")]
    AlreadyExists { id: u64 },
}

/// Handshake errors
#[derive(Debug, thiserror::Error)]
pub enum HandshakeError {
    /// Invalid request method
    #[error("Invalid HTTP method: {method}")]
    InvalidMethod { method: String },

    /// Invalid WebSocket version
    #[error("Invalid WebSocket version: {version}")]
    InvalidVersion { version: String },

    /// Missing required header
    #[error("Missing required header: {header}")]
    MissingHeader { header: String },

    /// Invalid header value
    #[error("Invalid header value for '{header}': {value}")]
    InvalidHeaderValue { header: String, value: String },

    /// Unsupported subprotocol
    #[error("Unsupported subprotocol: {protocol}")]
    UnsupportedSubprotocol { protocol: String },

    /// Unsupported extension
    #[error("Unsupported extension: {extension}")]
    UnsupportedExtension { extension: String },

    /// Invalid origin
    #[error("Invalid origin: {origin}")]
    InvalidOrigin { origin: String },

    /// Key mismatch
    #[error("WebSocket key mismatch")]
    KeyMismatch,

    /// Authentication failed
    #[error("Handshake authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
}

/// Protocol errors
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// Invalid opcode
    #[error("Invalid WebSocket opcode: {opcode}")]
    InvalidOpcode { opcode: u8 },

    /// Invalid frame format
    #[error("Invalid frame format: {reason}")]
    InvalidFrame { reason: String },

    /// Frame too large
    #[error("Frame too large: {size}/{max_size}")]
    FrameTooLarge { size: usize, max_size: usize },

    /// Message too large
    #[error("Message too large: {size}/{max_size}")]
    MessageTooLarge { size: usize, max_size: usize },

    /// Invalid UTF-8
    #[error("Invalid UTF-8 sequence in text frame")]
    InvalidUtf8,

    /// Control frame fragmented
    #[error("Control frame fragmented")]
    FragmentedControlFrame,

    /// Invalid continuation
    #[error("Invalid continuation frame")]
    InvalidContinuation,

    /// Masking required
    #[error("Client frames must be masked")]
    MaskingRequired,

    /// Masking forbidden
    #[error("Server frames must not be masked")]
    MaskingForbidden,

    /// Reserved bits set
    #[error("Reserved bits are set")]
    ReservedBitsSet,
}

/// Transport errors
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// Accept failed
    #[error("Failed to accept connection: {0}")]
    AcceptFailed(String),

    /// Bind failed
    #[error("Failed to bind to address: {0}")]
    BindFailed(String),

    /// Read failed
    #[error("Read failed: {0}")]
    ReadFailed(String),

    /// Write failed
    #[error("Write failed: {0}")]
    WriteFailed(String),

    /// Flush failed
    #[error("Flush failed: {0}")]
    FlushFailed(String),

    /// Close failed
    #[error("Close failed: {0}")]
    CloseFailed(String),

    /// TLS error
    #[error("TLS error: {0}")]
    Tls(String),
}

/// Handler errors
#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    /// Handler panicked
    #[error("Handler panicked: {0}")]
    Panicked(String),

    /// Handler returned error
    #[error("Handler returned error: {0}")]
    ReturnedError(String),

    /// Handler timeout
    #[error("Handler timed out after {duration:?}")]
    Timeout { duration: std::time::Duration },

    /// Handler not found
    #[error("Handler not found for path: {path}")]
    NotFound { path: String },
}

/// Manager errors
#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    /// Manager not initialized
    #[error("Connection manager not initialized")]
    NotInitialized,

    /// Manager shutdown
    #[error("Connection manager is shutting down")]
    Shutdown,

    /// Invalid connection ID
    #[error("Invalid connection ID: {id}")]
    InvalidId { id: u64 },

    /// Operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

impl From<ServerError> for Error {
    fn from(err: ServerError) -> Self {
        Error::Other(err.to_string())
    }
}

/// Error context for better error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Connection ID
    pub connection_id: Option<u64>,
    /// Remote address
    pub remote_addr: Option<std::net::SocketAddr>,
    /// Operation being performed
    pub operation: Option<String>,
    /// Additional context
    pub context: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self {
            connection_id: None,
            remote_addr: None,
            operation: None,
            context: std::collections::HashMap::new(),
        }
    }

    /// Set connection ID
    pub fn with_connection_id(mut self, id: u64) -> Self {
        self.connection_id = Some(id);
        self
    }

    /// Set remote address
    pub fn with_remote_addr(mut self, addr: std::net::SocketAddr) -> Self {
        self.remote_addr = Some(addr);
        self
    }

    /// Set operation
    pub fn with_operation(mut self, op: impl Into<String>) -> Self {
        self.operation = Some(op.into());
        self
    }

    /// Add context key-value pair
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type with error context
pub type ContextResult<T> = Result<T, ContextError>;

/// Error with context
#[derive(Debug)]
pub struct ContextError {
    /// The underlying error
    pub error: ServerError,
    /// Error context
    pub context: ErrorContext,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;
        
        if let Some(id) = self.context.connection_id {
            write!(f, " (connection: {})", id)?;
        }
        
        if let Some(addr) = self.context.remote_addr {
            write!(f, " (remote: {})", addr)?;
        }
        
        if let Some(op) = &self.context.operation {
            write!(f, " (operation: {})", op)?;
        }
        
        if !self.context.context.is_empty() {
            write!(f, " (context: {:?})", self.context.context)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Logging utilities
#[cfg(feature = "logging")]
pub mod logging {
    use super::*;
    use tracing::{error, warn, info, debug, trace, instrument, Level};

    /// Initialize logging with default configuration
    pub fn init_default() {
        tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .init();
    }

    /// Initialize logging with custom level
    pub fn init_with_level(level: Level) {
        tracing_subscriber::fmt()
            .with_max_level(level)
            .init();
    }

    /// Log server startup
    #[instrument]
    pub fn log_server_start(config: &crate::config::ServerConfig) {
        info!(
            bind_address = %config.bind_address,
            max_connections = config.max_connections,
            max_frame_size = config.max_frame_size,
            max_message_size = config.max_message_size,
            handshake_timeout = ?config.handshake_timeout,
            idle_timeout = ?config.idle_timeout,
            "WebSocket server starting"
        );
    }

    /// Log server shutdown
    #[instrument]
    pub fn log_server_shutdown() {
        info!("WebSocket server shutting down");
    }

    /// Log connection established
    #[instrument]
    pub fn log_connection_established(id: u64, remote_addr: std::net::SocketAddr) {
        info!(
            connection_id = id,
            remote_addr = %remote_addr,
            "WebSocket connection established"
        );
    }

    /// Log connection closed
    #[instrument]
    pub fn log_connection_closed(id: u64, reason: &str) {
        info!(
            connection_id = id,
            reason = reason,
            "WebSocket connection closed"
        );
    }

    /// Log message received
    #[instrument]
    pub fn log_message_received(id: u64, message_type: &str, size: usize) {
        trace!(
            connection_id = id,
            message_type = message_type,
            size = size,
            "Message received"
        );
    }

    /// Log message sent
    #[instrument]
    pub fn log_message_sent(id: u64, message_type: &str, size: usize) {
        trace!(
            connection_id = id,
            message_type = message_type,
            size = size,
            "Message sent"
        );
    }

    /// Log error with context
    #[instrument]
    pub fn log_error(error: &ContextError) {
        error!(
            error = %error,
            connection_id = ?error.context.connection_id,
            remote_addr = ?error.context.remote_addr,
            operation = ?error.context.operation,
            "Server error occurred"
        );
    }

    /// Log warning with context
    #[instrument]
    pub fn log_warning(message: &str, context: &ErrorContext) {
        warn!(
            message = message,
            connection_id = ?context.connection_id,
            remote_addr = ?context.remote_addr,
            operation = ?context.operation,
            "Server warning"
        );
    }

    /// Log performance metrics
    #[instrument]
    pub fn log_performance_metrics(stats: &crate::manager::ManagerStats) {
        info!(
            active_connections = stats.active_connections,
            total_connections = stats.total_connections,
            timeout_closures = stats.timeout_closures,
            error_closures = stats.error_closures,
            normal_closures = stats.normal_closures,
            peak_connections = stats.peak_connections,
            "Performance metrics"
        );
    }
}

/// Fallback logging when logging feature is disabled
#[cfg(not(feature = "logging"))]
pub mod logging {
    use super::*;

    /// No-op logging initialization
    pub fn init_default() {
        // No-op
    }

    /// No-op logging initialization with level
    pub fn init_with_level(_level: log::Level) {
        // No-op
    }

    /// No-op server start logging
    pub fn log_server_start(_config: &crate::config::ServerConfig) {
        // No-op
    }

    /// No-op server shutdown logging
    pub fn log_server_shutdown() {
        // No-op
    }

    /// No-op connection established logging
    pub fn log_connection_established(_id: u64, _remote_addr: std::net::SocketAddr) {
        // No-op
    }

    /// No-op connection closed logging
    pub fn log_connection_closed(_id: u64, _reason: &str) {
        // No-op
    }

    /// No-op message received logging
    pub fn log_message_received(_id: u64, _message_type: &str, _size: usize) {
        // No-op
    }

    /// No-op message sent logging
    pub fn log_message_sent(_id: u64, _message_type: &str, _size: usize) {
        // No-op
    }

    /// No-op error logging
    pub fn log_error(_error: &ContextError) {
        // No-op
    }

    /// No-op warning logging
    pub fn log_warning(_message: &str, _context: &ErrorContext) {
        // No-op
    }

    /// No-op performance metrics logging
    pub fn log_performance_metrics(_stats: &crate::manager::ManagerStats) {
        // No-op
    }
}
