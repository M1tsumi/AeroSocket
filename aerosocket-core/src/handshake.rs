//! WebSocket handshake implementation
//!
//! This module provides the WebSocket handshake functionality as defined in RFC 6455.
//! It includes both client and server handshake logic.

use crate::error::{Error, ProtocolError};
use crate::protocol::constants::*;
use crate::protocol::http_status::*;
use crate::protocol::http_header::*;
use crate::protocol::http_value;
use crate::protocol::http_method;
use sha1::{Digest, Sha1};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;

/// WebSocket handshake request information
#[derive(Debug, Clone)]
pub struct HandshakeRequest {
    /// HTTP method (should be GET)
    pub method: String,
    /// Request URI
    pub uri: String,
    /// HTTP version
    pub version: String,
    /// HTTP headers
    pub headers: HashMap<String, String>,
    /// Request body (should be empty for WebSocket handshake)
    pub body: Vec<u8>,
}

/// WebSocket handshake response information
#[derive(Debug, Clone)]
pub struct HandshakeResponse {
    /// HTTP status code
    pub status: u16,
    /// HTTP status message
    pub status_message: String,
    /// HTTP headers
    pub headers: HashMap<String, String>,
    /// Response body (should be empty for WebSocket handshake)
    pub body: Vec<u8>,
}

/// WebSocket handshake configuration
#[derive(Debug, Clone)]
pub struct HandshakeConfig {
    /// WebSocket protocols to offer/accept
    pub protocols: Vec<String>,
    /// WebSocket extensions to offer/accept
    pub extensions: Vec<String>,
    /// Origin to check against (server only)
    pub origin: Option<String>,
    /// Host header value (client only)
    pub host: Option<String>,
    /// Additional headers
    pub extra_headers: HashMap<String, String>,
}

impl Default for HandshakeConfig {
    fn default() -> Self {
        Self {
            protocols: vec![],
            extensions: vec![],
            origin: None,
            host: None,
            extra_headers: HashMap::new(),
        }
    }
}

/// Generate a random WebSocket key
pub fn generate_key() -> String {
    use rand::RngCore;
    let mut key_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    general_purpose::STANDARD.encode(key_bytes)
}

/// Compute WebSocket accept key from client key
pub fn compute_accept_key(client_key: &str) -> Result<String, Error> {
    let combined = format!("{}{}", client_key, WEBSOCKET_MAGIC);
    let hash = Sha1::digest(combined.as_bytes());
    Ok(general_purpose::STANDARD.encode(hash))
}

/// Validate WebSocket key format
pub fn validate_key(key: &str) -> bool {
    key.len() == 24 && general_purpose::STANDARD.decode(key).is_ok()
}

/// Validate WebSocket version
pub fn validate_version(version: &str) -> bool {
    version == WEBSOCKET_VERSION
}

/// Create a client handshake request
pub fn create_client_handshake(uri: &str, config: &HandshakeConfig) -> Result<HandshakeRequest, Error> {
    let mut headers = HashMap::new();
    
    // Required headers
    headers.insert(HEADER_UPGRADE.to_string(), http_value::WEBSOCKET.to_string());
    headers.insert(HEADER_CONNECTION.to_string(), http_value::UPGRADE.to_string());
    headers.insert(HEADER_SEC_WEBSOCKET_KEY.to_string(), generate_key());
    headers.insert(HEADER_SEC_WEBSOCKET_VERSION.to_string(), WEBSOCKET_VERSION.to_string());
    
    // Optional headers
    if let Some(host) = &config.host {
        headers.insert(HOST.to_string(), host.clone());
    }
    
    if let Some(origin) = &config.origin {
        headers.insert(ORIGIN.to_string(), origin.clone());
    }
    
    if !config.protocols.is_empty() {
        headers.insert(HEADER_SEC_WEBSOCKET_PROTOCOL.to_string(), config.protocols.join(", "));
    }
    
    if !config.extensions.is_empty() {
        headers.insert(HEADER_SEC_WEBSOCKET_EXTENSIONS.to_string(), config.extensions.join(", "));
    }
    
    // Add extra headers
    for (key, value) in &config.extra_headers {
        headers.insert(key.clone(), value.clone());
    }
    
    Ok(HandshakeRequest {
        method: http_method::GET.to_string(),
        uri: uri.to_string(),
        version: "HTTP/1.1".to_string(),
        headers,
        body: vec![],
    })
}

/// Parse a client handshake request
pub fn parse_client_handshake(request: &str) -> Result<HandshakeRequest, Error> {
    let mut lines = request.lines();
    
    // Parse request line
    let request_line = lines.next().ok_or_else(|| {
        Error::Protocol(ProtocolError::InvalidFormat("Missing request line".to_string()))
    })?;
    
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or_else(|| {
        Error::Protocol(ProtocolError::InvalidFormat("Missing method".to_string()))
    })?.to_string();
    
    let uri = parts.next().ok_or_else(|| {
        Error::Protocol(ProtocolError::InvalidFormat("Missing URI".to_string()))
    })?.to_string();
    
    let version = parts.next().ok_or_else(|| {
        Error::Protocol(ProtocolError::InvalidFormat("Missing HTTP version".to_string()))
    })?.to_string();
    
    // Validate method
    if method != http_method::GET {
        return Err(Error::Protocol(ProtocolError::InvalidMethod(method)));
    }
    
    // Parse headers
    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            break; // End of headers
        }
        
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        } else {
            return Err(Error::Protocol(ProtocolError::InvalidHeader {
                header: "unknown".to_string(),
                value: line.to_string(),
            }));
        }
    }
    
    Ok(HandshakeRequest {
        method,
        uri,
        version,
        headers,
        body: vec![],
    })
}

/// Validate a client handshake request
pub fn validate_client_handshake(request: &HandshakeRequest, config: &HandshakeConfig) -> Result<(), Error> {
    // Check required headers
    let upgrade = request.headers.get(HEADER_UPGRADE)
        .ok_or_else(|| Error::Protocol(ProtocolError::MissingHeader(HEADER_UPGRADE.to_string())))?;
    
    if upgrade.to_lowercase() != http_value::WEBSOCKET {
        return Err(Error::Protocol(ProtocolError::InvalidHeaderValue {
            header: HEADER_UPGRADE.to_string(),
            value: upgrade.clone(),
        }));
    }
    
    let connection = request.headers.get(HEADER_CONNECTION)
        .ok_or_else(|| Error::Protocol(ProtocolError::MissingHeader(HEADER_CONNECTION.to_string())))?;
    
    if !connection.to_lowercase().contains("upgrade") {
        return Err(Error::Protocol(ProtocolError::InvalidHeaderValue {
            header: HEADER_CONNECTION.to_string(),
            value: connection.clone(),
        }));
    }
    
    let key = request.headers.get(HEADER_SEC_WEBSOCKET_KEY)
        .ok_or_else(|| Error::Protocol(ProtocolError::MissingHeader(HEADER_SEC_WEBSOCKET_KEY.to_string())))?;
    
    if !validate_key(key) {
        return Err(Error::Protocol(ProtocolError::InvalidHeaderValue {
            header: HEADER_SEC_WEBSOCKET_KEY.to_string(),
            value: key.clone(),
        }));
    }
    
    let version = request.headers.get(HEADER_SEC_WEBSOCKET_VERSION)
        .ok_or_else(|| Error::Protocol(ProtocolError::MissingHeader(HEADER_SEC_WEBSOCKET_VERSION.to_string())))?;
    
    if !validate_version(version) {
        return Err(Error::Protocol(ProtocolError::InvalidHeaderValue {
            header: HEADER_SEC_WEBSOCKET_VERSION.to_string(),
            value: version.clone(),
        }));
    }
    
    // Check optional headers
    if let Some(origin) = &config.origin {
        if let Some(client_origin) = request.headers.get(ORIGIN) {
            if client_origin != origin {
                return Err(Error::Protocol(ProtocolError::InvalidOrigin {
                    expected: origin.clone(),
                    received: client_origin.clone(),
                }));
            }
        }
    }
    
    if !config.protocols.is_empty() {
        if let Some(protocol_header) = request.headers.get(HEADER_SEC_WEBSOCKET_PROTOCOL) {
            let client_protocols: Vec<&str> = protocol_header.split(',').map(|s| s.trim()).collect();
            if !client_protocols.iter().any(|p| config.protocols.contains(&p.to_string())) {
                return Err(Error::Protocol(ProtocolError::UnsupportedProtocol(protocol_header.clone())));
            }
        } else {
            return Err(Error::Protocol(ProtocolError::MissingHeader(HEADER_SEC_WEBSOCKET_PROTOCOL.to_string())));
        }
    }
    
    Ok(())
}

/// Create a server handshake response
pub fn create_server_handshake(request: &HandshakeRequest, config: &HandshakeConfig) -> Result<HandshakeResponse, Error> {
    let mut headers = HashMap::new();
    
    // Required headers
    headers.insert(HEADER_UPGRADE.to_string(), http_value::WEBSOCKET.to_string());
    headers.insert(HEADER_CONNECTION.to_string(), http_value::UPGRADE.to_string());
    
    // Compute accept key
    if let Some(client_key) = request.headers.get(HEADER_SEC_WEBSOCKET_KEY) {
        let accept_key = compute_accept_key(client_key)?;
        headers.insert(HEADER_SEC_WEBSOCKET_ACCEPT.to_string(), accept_key);
    } else {
        return Err(Error::Protocol(ProtocolError::MissingHeader(HEADER_SEC_WEBSOCKET_KEY.to_string())));
    }
    
    // Protocol negotiation
    if !config.protocols.is_empty() {
        if let Some(protocol_header) = request.headers.get(HEADER_SEC_WEBSOCKET_PROTOCOL) {
            let client_protocols: Vec<&str> = protocol_header.split(',').map(|s| s.trim()).collect();
            for protocol in &config.protocols {
                if client_protocols.contains(&protocol.as_str()) {
                    headers.insert(HEADER_SEC_WEBSOCKET_PROTOCOL.to_string(), protocol.clone());
                    break;
                }
            }
        }
    }
    
    // Add extra headers
    for (key, value) in &config.extra_headers {
        headers.insert(key.clone(), value.clone());
    }
    
    Ok(HandshakeResponse {
        status: SWITCHING_PROTOCOLS,
        status_message: "Switching Protocols".to_string(),
        headers,
        body: vec![],
    })
}

/// Parse a server handshake response
pub fn parse_server_handshake(response: &str) -> Result<HandshakeResponse, Error> {
    let mut lines = response.lines();
    
    // Parse status line
    let status_line = lines.next().ok_or_else(|| {
        Error::Protocol(ProtocolError::InvalidFormat("Missing status line".to_string()))
    })?;
    
    let mut parts = status_line.split_whitespace();
    let _version = parts.next().ok_or_else(|| {
        Error::Protocol(ProtocolError::InvalidFormat("Missing HTTP version".to_string()))
    })?.to_string();
    
    let status_str = parts.next().ok_or_else(|| {
        Error::Protocol(ProtocolError::InvalidFormat("Missing status code".to_string()))
    })?;
    
    let status = status_str.parse::<u16>().map_err(|_| {
        Error::Protocol(ProtocolError::InvalidFormat("Invalid status code".to_string()))
    })?;
    
    let status_message = parts.collect::<Vec<&str>>().join(" ");
    
    // Parse headers
    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            break; // End of headers
        }
        
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        } else {
            return Err(Error::Protocol(ProtocolError::InvalidHeader {
                header: "unknown".to_string(),
                value: line.to_string(),
            }));
        }
    }
    
    Ok(HandshakeResponse {
        status,
        status_message,
        headers,
        body: vec![],
    })
}

/// Validate a server handshake response
pub fn validate_server_handshake(response: &HandshakeResponse, client_key: &str) -> Result<(), Error> {
    // Check status code
    if response.status != SWITCHING_PROTOCOLS {
        return Err(Error::Protocol(ProtocolError::UnexpectedStatus(response.status)));
    }
    
    // Check required headers
    let upgrade = response.headers.get(HEADER_UPGRADE)
        .ok_or_else(|| Error::Protocol(ProtocolError::MissingHeader(HEADER_UPGRADE.to_string())))?;
    
    if upgrade.to_lowercase() != http_value::WEBSOCKET {
        return Err(Error::Protocol(ProtocolError::InvalidHeaderValue {
            header: HEADER_UPGRADE.to_string(),
            value: upgrade.clone(),
        }));
    }
    
    let connection = response.headers.get(HEADER_CONNECTION)
        .ok_or_else(|| Error::Protocol(ProtocolError::MissingHeader(HEADER_CONNECTION.to_string())))?;
    
    if !connection.to_lowercase().contains("upgrade") {
        return Err(Error::Protocol(ProtocolError::InvalidHeaderValue {
            header: HEADER_CONNECTION.to_string(),
            value: connection.clone(),
        }));
    }
    
    let accept = response.headers.get(HEADER_SEC_WEBSOCKET_ACCEPT)
        .ok_or_else(|| Error::Protocol(ProtocolError::MissingHeader(HEADER_SEC_WEBSOCKET_ACCEPT.to_string())))?;
    
    let expected_accept = compute_accept_key(client_key)?;
    if accept.as_str() != expected_accept {
        return Err(Error::Protocol(ProtocolError::InvalidAcceptKey {
            expected: expected_accept,
            received: accept.clone(),
        }));
    }
    
    Ok(())
}

/// Convert handshake request to HTTP string
pub fn request_to_string(request: &HandshakeRequest) -> String {
    let mut lines = vec![
        format!("{} {} {}", request.method, request.uri, request.version),
    ];
    
    for (key, value) in &request.headers {
        lines.push(format!("{}: {}", key, value));
    }
    
    lines.push(String::new()); // Empty line after headers
    lines.join("\r\n")
}

/// Convert handshake response to HTTP string
pub fn response_to_string(response: &HandshakeResponse) -> String {
    let mut lines = vec![
        format!("HTTP/1.1 {} {}", response.status, response.status_message),
    ];
    
    for (key, value) in &response.headers {
        lines.push(format!("{}: {}", key, value));
    }
    
    lines.push(String::new()); // Empty line after headers
    lines.join("\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = generate_key();
        assert_eq!(key.len(), 24);
        assert!(validate_key(&key));
    }

    #[test]
    fn test_accept_key_calculation() {
        let key = "dGhlIHNhbXBsZSBub25jZQ=="; // "the sample nonce"
        let expected = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
        let accept = compute_accept_key(key).unwrap();
        assert_eq!(accept, expected);
    }

    #[test]
    fn test_client_handshake_creation() {
        let config = HandshakeConfig {
            host: Some("example.com".to_string()),
            protocols: vec!["chat".to_string()],
            ..Default::default()
        };
        
        let request = create_client_handshake("ws://example.com/chat", &config).unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.uri, "ws://example.com/chat");
        assert_eq!(request.headers.get("upgrade").unwrap(), "websocket");
        assert_eq!(request.headers.get("sec-websocket-protocol").unwrap(), "chat");
    }

    #[test]
    fn test_client_handshake_parsing() {
        let raw_request = r#"GET /chat HTTP/1.1
Host: example.com
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13

"#;
        
        let request = parse_client_handshake(raw_request).unwrap();
        assert_eq!(request.method, "GET");
        assert_eq!(request.uri, "/chat");
        assert_eq!(request.headers.get("upgrade").unwrap(), "websocket");
    }
}
