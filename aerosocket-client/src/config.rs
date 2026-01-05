//! Client configuration for AeroSocket
//!
//! This module provides configuration options for WebSocket clients.

use aerosocket_core::error::ConfigError;
use aerosocket_core::Error;
use std::time::Duration;

#[cfg(feature = "transport-tls")]
use rustls::{
    Certificate, ClientConfig as RustlsClientConfig, OwnedTrustAnchor, PrivateKey, ProtocolVersion,
    RootCertStore,
};
#[cfg(feature = "transport-tls")]
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
#[cfg(feature = "transport-tls")]
use std::fs::File;
#[cfg(feature = "transport-tls")]
use std::io::BufReader;
#[cfg(feature = "transport-tls")]
use std::sync::Arc;
#[cfg(feature = "transport-tls")]
use webpki_roots::TLS_SERVER_ROOTS;

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
    /// Authentication
    pub auth: Option<aerosocket_core::Auth>,
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
            auth: None,
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

    /// Set authentication
    pub fn auth(mut self, auth: aerosocket_core::Auth) -> Self {
        self.auth = Some(auth);
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

#[cfg(feature = "transport-tls")]
fn load_certs(path: &str) -> aerosocket_core::Result<Vec<Certificate>> {
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
    Ok(cert_vec.into_iter().map(Certificate).collect())
}

#[cfg(feature = "transport-tls")]
fn load_private_key(path: &str) -> aerosocket_core::Result<PrivateKey> {
    let file = File::open(path).map_err(|e| {
        Error::Config(ConfigError::Validation(format!(
            "Failed to open private key file {}: {}",
            path, e
        )))
    })?;
    let mut reader = BufReader::new(file);

    // Try PKCS8 first
    if let Ok(keys) = pkcs8_private_keys(&mut reader) {
        if let Some(key) = keys.into_iter().next() {
            return Ok(PrivateKey(key));
        }
    }

    // Rewind and try RSA
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
        Ok(PrivateKey(key))
    } else {
        Err(Error::Config(ConfigError::Validation(format!(
            "No private keys found in {}",
            path
        ))))
    }
}

#[cfg(feature = "transport-tls")]
#[allow(dead_code)]
fn map_tls_version(v: TlsVersion) -> ProtocolVersion {
    match v {
        TlsVersion::V1_0 => ProtocolVersion::TLSv1_0,
        TlsVersion::V1_1 => ProtocolVersion::TLSv1_1,
        TlsVersion::V1_2 => ProtocolVersion::TLSv1_2,
        TlsVersion::V1_3 => ProtocolVersion::TLSv1_3,
    }
}

#[cfg(feature = "transport-tls")]
struct NoCertificateVerification;

#[cfg(feature = "transport-tls")]
impl rustls::client::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> std::result::Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

/// Build a rustls ClientConfig from this TlsConfig
#[cfg(feature = "transport-tls")]
#[allow(deprecated)]
pub fn build_rustls_client_config(tls: &TlsConfig) -> aerosocket_core::Result<RustlsClientConfig> {
    // Root store: either custom CA file or system/webpki roots
    let mut root_store = RootCertStore::empty();

    if let Some(ca_path) = &tls.ca_file {
        let certs = load_certs(ca_path)?;
        for cert in certs {
            root_store.add(&cert).map_err(|e| {
                Error::Config(ConfigError::Validation(format!(
                    "Failed to add CA certificate from {}: {:?}",
                    ca_path, e
                )))
            })?;
        }
    } else {
        root_store.add_trust_anchors(TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));
    }

    let builder = RustlsClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store);

    let mut config = if let (Some(cert_path), Some(key_path)) = (&tls.cert_file, &tls.key_file) {
        let certs = load_certs(cert_path)?;
        let key = load_private_key(key_path)?;
        builder.with_single_cert(certs, key).map_err(|e| {
            Error::Config(ConfigError::Validation(format!(
                "Invalid client certificate/key: {}",
                e
            )))
        })?
    } else {
        builder.with_no_client_auth()
    };

    // Apply verify flag: when false, disable certificate verification
    if !tls.verify {
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification));
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_frame_size, 16 * 1024 * 1024); // 16MB
        assert_eq!(config.max_message_size, 64 * 1024 * 1024); // 64MB
    }

    #[test]
    fn test_client_config_validation() {
        let config = ClientConfig {
            max_frame_size: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = ClientConfig {
            max_frame_size: 1024,
            max_message_size: 512,
            ..Default::default()
        };
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
