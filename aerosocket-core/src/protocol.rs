//! WebSocket protocol constants and utilities
//!
//! This module contains the fundamental protocol definitions from RFC 6455,
//! including opcodes, frame header bits, and protocol constants.

/// WebSocket opcodes as defined in RFC 6455 Section 5.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// Continuation frame
    Continuation = 0x0,
    /// Text frame
    Text = 0x1,
    /// Binary frame
    Binary = 0x2,
    /// Reserved for future use
    Reserved3 = 0x3,
    /// Reserved for future use
    Reserved4 = 0x4,
    /// Reserved for future use
    Reserved5 = 0x5,
    /// Reserved for future use
    Reserved6 = 0x6,
    /// Reserved for future use
    Reserved7 = 0x7,
    /// Close frame
    Close = 0x8,
    /// Ping frame
    Ping = 0x9,
    /// Pong frame
    Pong = 0xA,
    /// Reserved for future use
    ReservedB = 0xB,
    /// Reserved for future use
    ReservedC = 0xC,
    /// Reserved for future use
    ReservedD = 0xD,
    /// Reserved for future use
    ReservedE = 0xE,
    /// Reserved for future use
    ReservedF = 0xF,
}

impl Opcode {
    /// Create an Opcode from a u8
    pub fn from(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(Opcode::Continuation),
            0x1 => Some(Opcode::Text),
            0x2 => Some(Opcode::Binary),
            0x8 => Some(Opcode::Close),
            0x9 => Some(Opcode::Ping),
            0xA => Some(Opcode::Pong),
            _ => None,
        }
    }

    /// Get the numeric value of the opcode
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// Check if this is a control opcode
    pub fn is_control(&self) -> bool {
        matches!(self, Opcode::Close | Opcode::Ping | Opcode::Pong)
    }

    /// Check if this is a data opcode
    pub fn is_data(&self) -> bool {
        matches!(self, Opcode::Text | Opcode::Binary | Opcode::Continuation)
    }

    /// Check if this is a reserved opcode
    pub fn is_reserved(&self) -> bool {
        matches!(
            self,
            Opcode::Reserved3
                | Opcode::Reserved4
                | Opcode::Reserved5
                | Opcode::Reserved6
                | Opcode::Reserved7
                | Opcode::ReservedB
                | Opcode::ReservedC
                | Opcode::ReservedD
                | Opcode::ReservedE
                | Opcode::ReservedF
        )
    }
}

/// WebSocket protocol constants
pub mod constants {
    /// WebSocket protocol version
    pub const WEBSOCKET_VERSION: &str = "13";

    /// WebSocket upgrade header
    pub const HEADER_UPGRADE: &str = "upgrade";

    /// WebSocket connection header
    pub const HEADER_CONNECTION: &str = "connection";

    /// WebSocket key header
    pub const HEADER_SEC_WEBSOCKET_KEY: &str = "sec-websocket-key";

    /// WebSocket version header
    pub const HEADER_SEC_WEBSOCKET_VERSION: &str = "sec-websocket-version";

    /// WebSocket protocol header
    pub const HEADER_SEC_WEBSOCKET_PROTOCOL: &str = "sec-websocket-protocol";

    /// WebSocket extensions header
    pub const HEADER_SEC_WEBSOCKET_EXTENSIONS: &str = "sec-websocket-extensions";

    /// WebSocket accept header
    pub const HEADER_SEC_WEBSOCKET_ACCEPT: &str = "sec-websocket-accept";

    /// WebSocket magic string for accept calculation
    pub const WEBSOCKET_MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    /// Maximum frame size (default)
    pub const DEFAULT_MAX_FRAME_SIZE: usize = 16 * 1024 * 1024; // 16MB

    /// Maximum message size (default)
    pub const DEFAULT_MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024; // 64MB

    /// Default handshake timeout
    pub const DEFAULT_HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

    /// Default idle timeout
    pub const DEFAULT_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300); // 5 minutes

    /// WebSocket key length in bytes
    pub const WEBSOCKET_KEY_LEN: usize = 16;

    /// WebSocket accept value length in bytes
    pub const WEBSOCKET_ACCEPT_LEN: usize = 28;

    /// Maximum header size
    pub const MAX_HEADER_SIZE: usize = 8192; // 8KB

    /// Minimum close frame payload size
    pub const MIN_CLOSE_PAYLOAD_SIZE: usize = 2;

    /// Maximum close reason size
    pub const MAX_CLOSE_REASON_SIZE: usize = 123;
}

/// Frame header bit positions and masks
pub mod frame {
    /// FIN bit position
    pub const FIN_BIT: u8 = 0x80;

    /// RSV1 bit position
    pub const RSV1_BIT: u8 = 0x40;

    /// RSV2 bit position
    pub const RSV2_BIT: u8 = 0x20;

    /// RSV3 bit position
    pub const RSV3_BIT: u8 = 0x10;

    /// Opcode mask
    pub const OPCODE_MASK: u8 = 0x0F;

    /// MASK bit position
    pub const MASK_BIT: u8 = 0x80;

    /// Payload length mask for 7-bit length
    pub const PAYLOAD_LEN_MASK: u8 = 0x7F;

    /// Extended payload length (16-bit) marker
    pub const PAYLOAD_LEN_16: u8 = 126;

    /// Extended payload length (64-bit) marker
    pub const PAYLOAD_LEN_64: u8 = 127;

    /// Masking key length
    pub const MASKING_KEY_LEN: usize = 4;
}

/// HTTP status codes used in WebSocket handshake
pub mod http_status {
    /// HTTP Switching Protocols status
    pub const SWITCHING_PROTOCOLS: u16 = 101;

    /// HTTP Bad Request status
    pub const BAD_REQUEST: u16 = 400;

    /// HTTP Unauthorized status
    pub const UNAUTHORIZED: u16 = 401;

    /// HTTP Forbidden status
    pub const FORBIDDEN: u16 = 403;

    /// HTTP Method Not Allowed status
    pub const METHOD_NOT_ALLOWED: u16 = 405;

    /// HTTP Upgrade Required status
    pub const UPGRADE_REQUIRED: u16 = 426;

    /// HTTP Internal Server Error status
    pub const INTERNAL_SERVER_ERROR: u16 = 500;

    /// HTTP Service Unavailable status
    pub const SERVICE_UNAVAILABLE: u16 = 503;
}

/// HTTP methods
pub mod http_method {
    /// HTTP GET method
    pub const GET: &str = "GET";

    /// HTTP HEAD method
    pub const HEAD: &str = "HEAD";

    /// HTTP POST method
    pub const POST: &str = "POST";

    /// HTTP PUT method
    pub const PUT: &str = "PUT";

    /// HTTP DELETE method
    pub const DELETE: &str = "DELETE";

    /// HTTP CONNECT method
    pub const CONNECT: &str = "CONNECT";

    /// HTTP OPTIONS method
    pub const OPTIONS: &str = "OPTIONS";

    /// HTTP TRACE method
    pub const TRACE: &str = "TRACE";

    /// HTTP PATCH method
    pub const PATCH: &str = "PATCH";
}

/// HTTP header names (lowercase for consistency)
pub mod http_header {
    /// Host header
    pub const HOST: &str = "host";

    /// User-Agent header
    pub const USER_AGENT: &str = "user-agent";

    /// Accept header
    pub const ACCEPT: &str = "accept";

    /// Connection header
    pub const CONNECTION: &str = "connection";

    /// Upgrade header
    pub const UPGRADE: &str = "upgrade";

    /// Origin header
    pub const ORIGIN: &str = "origin";

    /// Sec-WebSocket-Key header
    pub const SEC_WEBSOCKET_KEY: &str = "sec-websocket-key";

    /// Sec-WebSocket-Version header
    pub const SEC_WEBSOCKET_VERSION: &str = "sec-websocket-version";

    /// Sec-WebSocket-Protocol header
    pub const SEC_WEBSOCKET_PROTOCOL: &str = "sec-websocket-protocol";

    /// Sec-WebSocket-Extensions header
    pub const SEC_WEBSOCKET_EXTENSIONS: &str = "sec-websocket-extensions";

    /// Sec-WebSocket-Accept header
    pub const SEC_WEBSOCKET_ACCEPT: &str = "sec-websocket-accept";

    /// Authorization header
    pub const AUTHORIZATION: &str = "authorization";

    /// Cookie header
    pub const COOKIE: &str = "cookie";

    /// Set-Cookie header
    pub const SET_COOKIE: &str = "set-cookie";

    /// Content-Type header
    pub const CONTENT_TYPE: &str = "content-type";

    /// Content-Length header
    pub const CONTENT_LENGTH: &str = "content-length";

    /// Date header
    pub const DATE: &str = "date";

    /// Server header
    pub const SERVER: &str = "server";
}

/// HTTP header values
pub mod http_value {
    /// WebSocket upgrade value
    pub const WEBSOCKET: &str = "websocket";

    /// Upgrade connection value
    pub const UPGRADE: &str = "Upgrade";

    /// Keep-Alive connection value
    pub const KEEP_ALIVE: &str = "keep-alive";

    /// Close connection value
    pub const CLOSE: &str = "close";

    /// Application JSON content type
    pub const APPLICATION_JSON: &str = "application/json";

    /// Text plain content type
    pub const TEXT_PLAIN: &str = "text/plain";

    /// Application octet-stream content type
    pub const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";
}

/// WebSocket extension parameters
pub mod extensions {
    /// Per-message deflate extension
    pub const PERMESSAGE_DEFLATE: &str = "permessage-deflate";

    /// Per-message deflate client context takeover
    pub const CLIENT_CONTEXT_TAKEOVER: &str = "client_context_takeover";

    /// Per-message deflate server context takeover
    pub const SERVER_CONTEXT_TAKEOVER: &str = "server_context_takeover";

    /// Per-message deflate client max window bits
    pub const CLIENT_MAX_WINDOW_BITS: &str = "client_max_window_bits";

    /// Per-message deflate server max window bits
    pub const SERVER_MAX_WINDOW_BITS: &str = "server_max_window_bits";

    /// Per-message deflate client no context takeover
    pub const CLIENT_NO_CONTEXT_TAKEOVER: &str = "client_no_context_takeover";

    /// Per-message deflate server no context takeover
    pub const SERVER_NO_CONTEXT_TAKEOVER: &str = "server_no_context_takeover";
}

/// Utility functions for WebSocket protocol operations
pub mod utils {
    use crate::error::{Error, ProtocolError};
    use sha1::{Digest, Sha1};
    use base64::{engine::general_purpose, Engine as _};

    /// Generate a random WebSocket key
    pub fn generate_key() -> String {
        use rand::RngCore;
        let mut key_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key_bytes);
        general_purpose::STANDARD.encode(key_bytes)
    }

    /// Compute WebSocket accept key
    pub fn calculate_accept(key: &str) -> String {
        let combined = format!("{}{}", key, super::constants::WEBSOCKET_MAGIC);
        let hash = Sha1::digest(combined.as_bytes());
        general_purpose::STANDARD.encode(hash)
    }

    /// Validate WebSocket key format
    pub fn validate_key(key: &str) -> bool {
        key.len() == 24 && general_purpose::STANDARD.decode(key).is_ok()
    }

    /// Validate WebSocket version
    pub fn validate_version(version: &str) -> bool {
        version == super::constants::WEBSOCKET_VERSION
    }

    /// Check if a close code is valid
    pub fn is_valid_close_code(code: u16) -> bool {
        use crate::error::CloseCode;
        matches!(
            CloseCode::from(code),
            CloseCode::Normal
                | CloseCode::Away
                | CloseCode::ProtocolError
                | CloseCode::Unsupported
                | CloseCode::InvalidPayload
                | CloseCode::PolicyViolation
                | CloseCode::TooBig
                | CloseCode::MandatoryExtension
                | CloseCode::Internal
                | CloseCode::Application(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_conversion() {
        assert_eq!(Opcode::from(0x1), Some(Opcode::Text));
        assert_eq!(Opcode::from(0xFF), None);
        assert_eq!(Opcode::Text.value(), 0x1);
        assert!(Opcode::Ping.is_control());
        assert!(Opcode::Binary.is_data());
        assert!(Opcode::Reserved3.is_reserved());
    }

    #[test]
    fn test_websocket_key_generation() {
        let key = utils::generate_key();
        assert!(utils::validate_key(&key));
    }

    #[test]
    fn test_websocket_accept_calculation() {
        let key = "dGhlIHNhbXBsZSBub25jZQ=="; // "the sample nonce"
        let expected = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
        assert_eq!(utils::calculate_accept(key), expected);
    }

    #[test]
    fn test_close_code_validation() {
        assert!(utils::is_valid_close_code(1000));
        assert!(utils::is_valid_close_code(3000));
        assert!(!utils::is_valid_close_code(999));
    }
}
