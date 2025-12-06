//! Server configuration
//!
//! This module provides configuration options for the WebSocket server.

use aerosocket_core::error::{ConfigError, Error};
use std::time::Duration;

#[cfg(feature = "tls-transport")]
use rustls::{Certificate as RustlsCert, PrivateKey as RustlsKey, ServerConfig as RustlsServerConfig};
#[cfg(feature = "tls-transport")]
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
#[cfg(feature = "tls-transport")]
use std::fs::File;
#[cfg(feature = "tls-transport")]
use std::io::BufReader;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address
    pub bind_address: std::net::SocketAddr,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Maximum frame size in bytes
    pub max_frame_size: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Handshake timeout
    pub handshake_timeout: Duration,
    /// Idle timeout
    pub idle_timeout: Duration,
    /// Compression configuration
    pub compression: CompressionConfig,
    /// Backpressure configuration
    pub backpressure: BackpressureConfig,
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    /// Transport type
    pub transport_type: TransportType,
    /// Supported WebSocket subprotocols
    pub supported_protocols: Vec<String>,
    /// Supported WebSocket extensions
    pub supported_extensions: Vec<String>,
    /// Allowed origin (for CORS)
    pub allowed_origin: Option<String>,
    /// Extra headers to send in handshake response
    pub extra_headers: std::collections::HashMap<String, String>,
}

/// Transport type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// TCP transport
    Tcp,
    /// TLS transport
    Tls,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8080".parse().unwrap(),
            max_connections: 10_000,
            max_frame_size: aerosocket_core::protocol::constants::DEFAULT_MAX_FRAME_SIZE,
            max_message_size: aerosocket_core::protocol::constants::DEFAULT_MAX_MESSAGE_SIZE,
            handshake_timeout: aerosocket_core::protocol::constants::DEFAULT_HANDSHAKE_TIMEOUT,
            idle_timeout: aerosocket_core::protocol::constants::DEFAULT_IDLE_TIMEOUT,
            compression: CompressionConfig::default(),
            backpressure: BackpressureConfig::default(),
            tls: None,
            transport_type: TransportType::Tcp,
            supported_protocols: vec![],
            supported_extensions: vec![],
            allowed_origin: None,
            extra_headers: std::collections::HashMap::new(),
        }
    }
}

impl ServerConfig {
    /// Validate the configuration
    pub fn validate(&self) -> aerosocket_core::Result<()> {
        if self.max_connections == 0 {
            return Err(Error::Config(ConfigError::Validation(
                "max_connections must be greater than 0".to_string(),
            )));
        }

        if self.max_frame_size == 0 {
            return Err(Error::Config(ConfigError::Validation(
                "max_frame_size must be greater than 0".to_string(),
            )));
        }

        if self.max_message_size == 0 {
            return Err(Error::Config(ConfigError::Validation(
                "max_message_size must be greater than 0".to_string(),
            )));
        }

        if self.max_message_size < self.max_frame_size {
            return Err(Error::Config(ConfigError::Validation(
                "max_message_size must be greater than or equal to max_frame_size".to_string(),
            )));
        }

        Ok(())
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    /// Compression level (0-9)
    pub level: u8,
    /// Server context takeover
    pub server_context_takeover: bool,
    /// Client context takeover
    pub client_context_takeover: bool,
    /// Server max window bits
    pub server_max_window_bits: Option<u8>,
    /// Client max window bits
    pub client_max_window_bits: Option<u8>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: 6,
            server_context_takeover: true,
            client_context_takeover: true,
            server_max_window_bits: None,
            client_max_window_bits: None,
        }
    }
}

/// Backpressure configuration
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Whether backpressure is enabled
    pub enabled: bool,
    /// Maximum requests per minute per IP
    pub max_requests_per_minute: usize,
    /// Backpressure strategy
    pub strategy: BackpressureStrategy,
    /// Buffer size in bytes
    pub buffer_size: usize,
    /// High water mark in bytes
    pub high_water_mark: usize,
    /// Low water mark in bytes
    pub low_water_mark: usize,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_minute: 60,
            strategy: BackpressureStrategy::Buffer,
            buffer_size: 64 * 1024,     // 64KB
            high_water_mark: 48 * 1024, // 48KB
            low_water_mark: 16 * 1024,  // 16KB
        }
    }
}

/// Backpressure strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureStrategy {
    /// Buffer messages (default)
    Buffer,
    /// Drop oldest messages when buffer is full
    DropOldest,
    /// Reject new messages when buffer is full
    Reject,
    /// Apply flow control to sender
    FlowControl,
}

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to certificate file
    pub cert_file: String,
    /// Path to private key file
    pub key_file: String,
    /// Certificate chain file (optional)
    pub cert_chain_file: Option<String>,
    /// Enable client authentication
    pub client_auth: bool,
    /// CA file for client authentication
    pub ca_file: Option<String>,
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new(cert_file: String, key_file: String) -> Self {
        Self {
            cert_file,
            key_file,
            cert_chain_file: None,
            client_auth: false,
            ca_file: None,
        }
    }

    /// Set certificate chain file
    pub fn cert_chain_file(mut self, file: String) -> Self {
        self.cert_chain_file = Some(file);
        self
    }

    /// Enable client authentication
    pub fn client_auth(mut self, enabled: bool) -> Self {
        self.client_auth = enabled;
        self
    }

    /// Set CA file for client authentication
    pub fn ca_file(mut self, file: String) -> Self {
        self.ca_file = Some(file);
        self
    }
}

#[cfg(feature = "tls-transport")]
fn load_certs(path: &str) -> aerosocket_core::Result<Vec<RustlsCert>> {
    let file = File::open(path).map_err(|e| {
        Error::Config(ConfigError::Validation(format!(
            "Failed to open certificate file {}: {}",
            path, e
        )))
    })?;
    let mut reader = BufReader::new(file);
    let cert_vec = certs(&mut reader).map_err(|e| {
        Error::Config(ConfigError::Validation(format!(
            "Failed to parse certificate file {}: {}",
            path, e
        )))
    })?;
    Ok(cert_vec.into_iter().map(RustlsCert).collect())
}

#[cfg(feature = "tls-transport")]
fn load_private_key(path: &str) -> aerosocket_core::Result<RustlsKey> {
    let file = File::open(path).map_err(|e| {
        Error::Config(ConfigError::Validation(format!(
            "Failed to open private key file {}: {}",
            path, e
        )))
    })?;
    let mut reader = BufReader::new(file);

    if let Ok(keys) = pkcs8_private_keys(&mut reader) {
        if let Some(key) = keys.into_iter().next() {
            return Ok(RustlsKey(key));
        }
    }

    let file = File::open(path).map_err(|e| {
        Error::Config(ConfigError::Validation(format!(
            "Failed to reopen private key file {}: {}",
            path, e
        )))
    })?;
    let mut reader = BufReader::new(file);
    let keys = rsa_private_keys(&mut reader).map_err(|e| {
        Error::Config(ConfigError::Validation(format!(
            "Failed to parse private key file {}: {}",
            path, e
        )))
    })?;

    if let Some(key) = keys.into_iter().next() {
        Ok(RustlsKey(key))
    } else {
        Err(Error::Config(ConfigError::Validation(format!(
            "No private keys found in {}",
            path
        ))))
    }
}

#[cfg(feature = "tls-transport")]
pub fn build_rustls_server_config(tls: &TlsConfig) -> aerosocket_core::Result<RustlsServerConfig> {
    let certs = load_certs(&tls.cert_file)?;
    let key = load_private_key(&tls.key_file)?;

    RustlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| {
            Error::Config(ConfigError::Validation(format!(
                "Invalid TLS certificate/key: {}",
                e
            )))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_connections, 10_000);
        assert_eq!(config.bind_address.port(), 8080);
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 1000;
        config.max_frame_size = 0;
        assert!(config.validate().is_err());

        config.max_frame_size = 1024;
        config.max_message_size = 512;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tls_config() {
        let config = TlsConfig::new("cert.pem".to_string(), "key.pem".to_string())
            .cert_chain_file("chain.pem".to_string())
            .client_auth(true)
            .ca_file("ca.pem".to_string());

        assert_eq!(config.cert_file, "cert.pem");
        assert_eq!(config.key_file, "key.pem");
        assert_eq!(config.cert_chain_file, Some("chain.pem".to_string()));
        assert!(config.client_auth);
        assert_eq!(config.ca_file, Some("ca.pem".to_string()));
    }
}
