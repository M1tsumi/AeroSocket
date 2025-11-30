//! Client configuration for AeroSocket
//!
//! This module provides configuration options for WebSocket clients.

use aerosocket_core::error::ConfigError;
use aerosocket_core::Error;
use std::time::Duration;

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
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
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    /// User agent string
    pub user_agent: String,
    /// Origin header
    pub origin: Option<String>,
    /// WebSocket subprotocols
    pub protocols: Vec<String>,
    /// Custom headers
    pub headers: Vec<(String, String)>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            max_frame_size: aerosocket_core::protocol::constants::DEFAULT_MAX_FRAME_SIZE,
            max_message_size: aerosocket_core::protocol::constants::DEFAULT_MAX_MESSAGE_SIZE,
            handshake_timeout: aerosocket_core::protocol::constants::DEFAULT_HANDSHAKE_TIMEOUT,
            idle_timeout: aerosocket_core::protocol::constants::DEFAULT_IDLE_TIMEOUT,
            compression: CompressionConfig::default(),
            tls: None,
            user_agent: format!("aerosocket-client/{}", env!("CARGO_PKG_VERSION")),
            origin: None,
            protocols: Vec::new(),
            headers: Vec::new(),
        }
    }
}

impl ClientConfig {
    /// Validate the configuration
    pub fn validate(&self) -> aerosocket_core::Result<()> {
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

        if self.handshake_timeout.is_zero() {
            return Err(Error::Config(ConfigError::Validation(
                "handshake_timeout must be greater than 0".to_string(),
            )));
        }

        Ok(())
    }

    /// Set maximum frame size
    pub fn max_frame_size(mut self, size: usize) -> Self {
        self.max_frame_size = size;
        self
    }

    /// Set maximum message size
    pub fn max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Set handshake timeout
    pub fn handshake_timeout(mut self, timeout: Duration) -> Self {
        self.handshake_timeout = timeout;
        self
    }

    /// Set idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, agent: String) -> Self {
        self.user_agent = agent;
        self
    }

    /// Set origin
    pub fn origin(mut self, origin: String) -> Self {
        self.origin = Some(origin);
        self
    }

    /// Add a subprotocol
    pub fn add_protocol(mut self, protocol: String) -> Self {
        self.protocols.push(protocol);
        self
    }

    /// Add a custom header
    pub fn add_header(mut self, name: String, value: String) -> Self {
        self.headers.push((name, value));
        self
    }

    /// Set TLS configuration
    pub fn tls(mut self, config: TlsConfig) -> Self {
        self.tls = Some(config);
        self
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

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Enable TLS verification
    pub verify: bool,
    /// Path to CA certificate file
    pub ca_file: Option<String>,
    /// Path to client certificate file
    pub cert_file: Option<String>,
    /// Path to client private key file
    pub key_file: Option<String>,
    /// Server name for SNI
    pub server_name: Option<String>,
    /// Minimum TLS version
    pub min_version: Option<TlsVersion>,
    /// Maximum TLS version
    pub max_version: Option<TlsVersion>,
}

/// TLS version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    /// TLS 1.0
    V1_0,
    /// TLS 1.1
    V1_1,
    /// TLS 1.2
    V1_2,
    /// TLS 1.3
    V1_3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_frame_size, 1024 * 1024);
        assert_eq!(config.max_message_size, 16 * 1024 * 1024);
    }

    #[test]
    fn test_client_config_validation() {
        let mut config = ClientConfig::default();
        config.max_frame_size = 0;
        assert!(config.validate().is_err());

        config.max_frame_size = 1024;
        config.max_message_size = 512;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_client_config_builder() {
        let config = ClientConfig::default()
            .max_frame_size(2048)
            .user_agent("test-agent".to_string())
            .add_protocol("chat".to_string())
            .add_header("X-Custom".to_string(), "value".to_string());

        assert_eq!(config.max_frame_size, 2048);
        assert_eq!(config.user_agent, "test-agent");
        assert_eq!(config.protocols.len(), 1);
        assert_eq!(config.protocols[0], "chat");
        assert_eq!(config.headers.len(), 1);
        assert_eq!(
            config.headers[0],
            ("X-Custom".to_string(), "value".to_string())
        );
    }
}
