//! Error types for AeroSocket
//!
//! This module defines all error types used throughout the AeroSocket library.
//! Errors are designed to be ergonomic and provide clear context for debugging.

#![allow(missing_docs)]
#![allow(clippy::recursive_format_impl)]

use std::fmt;
use thiserror::Error;

/// Result type alias for AeroSocket operations
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error type for AeroSocket operations
#[derive(Error, Debug)]
pub enum Error {
    /// Protocol errors
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Frame errors
    #[error("Frame error: {0}")]
    Frame(#[from] FrameError),

    /// Message errors
    #[error("Message error: {0}")]
    Message(#[from] MessageError),

    /// Security errors
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    Timeout(#[from] TimeoutError),

    /// Close errors
    #[error("Close error: {0}")]
    Close(#[from] CloseError),

    /// Connection errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Generic errors
    #[error("Error: {0}")]
    Other(String),

    /// Buffer capacity exceeded
    #[error("Buffer capacity exceeded: {size} bytes")]
    CapacityExceeded { size: usize },

    /// Invalid UTF-8 in text frame
    #[error("Invalid UTF-8 in text frame")]
    InvalidUtf8,

    /// Connection closed
    #[error("Connection closed: {code} - {reason}")]
    Closed {
        /// Close code
        code: CloseCode,
        /// Close reason
        reason: String,
    },
}

/// WebSocket protocol specific errors
#[derive(Error, Debug, Clone)]
pub enum ProtocolError {
    /// Invalid WebSocket version
    #[error("Unsupported WebSocket version")]
    UnsupportedVersion,

    /// Invalid upgrade request
    #[error("Invalid WebSocket upgrade request")]
    InvalidUpgradeRequest,

    /// Missing required headers
    #[error("Missing required header: {0}")]
    MissingHeader(String),

    /// Invalid header value
    #[error("Invalid header value for {header}: {value}")]
    InvalidHeader { header: String, value: String },

    /// Extension negotiation failed
    #[error("Extension negotiation failed: {0}")]
    ExtensionNegotiation(String),

    /// Subprotocol negotiation failed
    #[error("Subprotocol negotiation failed")]
    SubprotocolNegotiation,

    /// Invalid frame format
    #[error("Invalid frame format: {0}")]
    InvalidFrame(String),

    /// Control frame fragmentation not allowed
    #[error("Control frames cannot be fragmented")]
    FragmentedControlFrame,

    /// Invalid close code
    #[error("Invalid close code: {0}")]
    InvalidCloseCode(u16),

    /// Reserved bits set in frame
    #[error("Reserved bits set in frame")]
    ReservedBitsSet,

    /// Invalid HTTP method
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Invalid header value
    #[error("Invalid header value for {header}: {value}")]
    InvalidHeaderValue { header: String, value: String },

    /// Invalid origin
    #[error("Invalid origin - expected: {expected}, received: {received}")]
    InvalidOrigin { expected: String, received: String },

    /// Unsupported protocol
    #[error("Unsupported WebSocket protocol: {0}")]
    UnsupportedProtocol(String),

    /// Unexpected HTTP status
    #[error("Unexpected HTTP status: {0}")]
    UnexpectedStatus(u16),

    /// Invalid accept key
    #[error("Invalid WebSocket accept key - expected: {expected}, received: {received}")]
    InvalidAcceptKey { expected: String, received: String },
}

/// Frame parsing and processing errors
#[derive(Error, Debug, Clone)]
pub enum FrameError {
    /// Insufficient data to parse frame
    #[error("Insufficient data: need {needed} bytes, have {have}")]
    InsufficientData { needed: usize, have: usize },

    /// Frame too large
    #[error("Frame too large: {size} bytes (max: {max})")]
    TooLarge { size: usize, max: usize },

    /// Invalid frame header
    #[error("Invalid frame header: {0}")]
    InvalidHeader(String),

    /// Invalid masking
    #[error("Invalid masking: {0}")]
    InvalidMasking(String),

    /// Invalid opcode
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),

    /// Reserved bits set
    #[error("Reserved bits set in frame")]
    ReservedBitsSet,

    /// Decompression failed
    #[error("Decompression failed")]
    DecompressionFailed,

    /// Control frames cannot be fragmented
    #[error("Control frames cannot be fragmented")]
    FragmentedControlFrame,
}

/// Configuration errors
#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    /// Invalid configuration value
    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },

    /// Missing required configuration
    #[error("Missing required configuration: {field}")]
    MissingField { field: String },

    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    Validation(String),
}

/// Message errors
#[derive(Error, Debug, Clone)]
pub enum MessageError {
    /// Message too large
    #[error("Message too large: {size} bytes (max: {max})")]
    TooLarge { size: usize, max: usize },

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    /// Fragmented control message
    #[error("Control messages cannot be fragmented")]
    FragmentedControl,

    /// Incomplete message
    #[error("Incomplete message: missing {missing}")]
    Incomplete { missing: String },
}

/// Security errors
#[derive(Error, Debug, Clone)]
pub enum SecurityError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Authorization failed
    #[error("Authorization failed: {0}")]
    Authorization(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Blocked connection
    #[error("Connection blocked: {reason}")]
    Blocked { reason: String },

    /// Security policy violation
    #[error("Security policy violation: {0}")]
    PolicyViolation(String),
}

/// Timeout errors
#[derive(Error, Debug, Clone)]
pub enum TimeoutError {
    /// Handshake timeout
    #[error("Handshake timeout: {timeout:?}")]
    Handshake { timeout: std::time::Duration },

    /// Read timeout
    #[error("Read timeout: {timeout:?}")]
    Read { timeout: std::time::Duration },

    /// Write timeout
    #[error("Write timeout: {timeout:?}")]
    Write { timeout: std::time::Duration },

    /// Idle timeout
    #[error("Idle timeout: {timeout:?}")]
    Idle { timeout: std::time::Duration },
}

/// Close errors
#[derive(Error, Debug, Clone)]
pub enum CloseError {
    /// Invalid close code
    #[error("Invalid close code: {code}")]
    InvalidCode { code: u16 },

    /// Close reason too long
    #[error("Close reason too long: {len} bytes (max: {max})")]
    ReasonTooLong { len: usize, max: usize },

    /// UTF-8 error in close reason
    #[error("Invalid UTF-8 in close reason")]
    InvalidUtf8,
}

/// WebSocket close codes as defined in RFC 6455
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CloseCode {
    /// Normal closure
    Normal = 1000,

    /// Going away
    Away = 1001,

    /// Protocol error
    ProtocolError = 1002,

    /// Unsupported data
    Unsupported = 1003,

    /// No status received
    NoStatus = 1005,

    /// Abnormal closure
    Abnormal = 1006,

    /// Invalid frame payload data
    InvalidPayload = 1007,

    /// Policy violation
    PolicyViolation = 1008,

    /// Message too big
    TooBig = 1009,

    /// Mandatory extension
    MandatoryExtension = 1010,

    /// Internal server error
    Internal = 1011,

    /// TLS handshake failure
    TlsHandshake = 1015,

    /// Application-specific close code
    Application(u16) = 3000,
}

impl CloseCode {
    /// Create a CloseCode from a u16
    pub fn from(code: u16) -> Self {
        match code {
            1000 => CloseCode::Normal,
            1001 => CloseCode::Away,
            1002 => CloseCode::ProtocolError,
            1003 => CloseCode::Unsupported,
            1005 => CloseCode::NoStatus,
            1006 => CloseCode::Abnormal,
            1007 => CloseCode::InvalidPayload,
            1008 => CloseCode::PolicyViolation,
            1009 => CloseCode::TooBig,
            1010 => CloseCode::MandatoryExtension,
            1011 => CloseCode::Internal,
            1015 => CloseCode::TlsHandshake,
            code if (3000..=4999).contains(&code) => CloseCode::Application(code),
            _ => CloseCode::ProtocolError,
        }
    }

    /// Get the numeric value of the close code
    pub fn code(&self) -> u16 {
        match self {
            CloseCode::Normal => 1000,
            CloseCode::Away => 1001,
            CloseCode::ProtocolError => 1002,
            CloseCode::Unsupported => 1003,
            CloseCode::NoStatus => 1005,
            CloseCode::Abnormal => 1006,
            CloseCode::InvalidPayload => 1007,
            CloseCode::PolicyViolation => 1008,
            CloseCode::TooBig => 1009,
            CloseCode::MandatoryExtension => 1010,
            CloseCode::Internal => 1011,
            CloseCode::TlsHandshake => 1015,
            CloseCode::Application(code) => *code,
        }
    }

    /// Check if this is a reserved close code
    pub fn is_reserved(&self) -> bool {
        matches!(
            self,
            CloseCode::NoStatus | CloseCode::Abnormal | CloseCode::TlsHandshake
        )
    }

    /// Check if this close code indicates an error
    pub fn is_error(&self) -> bool {
        !matches!(self, CloseCode::Normal | CloseCode::Away)
    }
}

impl fmt::Display for CloseCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self, self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_code_conversion() {
        assert_eq!(CloseCode::from(1000), CloseCode::Normal);
        assert_eq!(CloseCode::from(3000), CloseCode::Application(3000));
        assert_eq!(CloseCode::from(999), CloseCode::ProtocolError);
    }

    #[test]
    fn test_error_display() {
        let err = Error::Protocol(ProtocolError::UnsupportedVersion);
        let msg = err.to_string();
        println!("Error message: {}", msg); // Debug output
        assert!(msg.contains("protocol") || msg.contains("version") || msg.contains("WebSocket"));
    }
}
