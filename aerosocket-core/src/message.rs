//! Message handling for AeroSocket
//!
//! This module provides high-level message types and handling for WebSocket messages,
//! including support for text, binary, ping, pong, and close messages.

use crate::error::{Error, Result, ProtocolError, CloseCode};
use crate::frame::{Frame, FrameKind};
use crate::protocol::Opcode;
use bytes::{Bytes, BytesMut};
use std::fmt;

/// Represents a complete WebSocket message
#[derive(Debug, Clone)]
pub enum Message {
    /// Text message
    Text(TextMessage),
    /// Binary message
    Binary(BinaryMessage),
    /// Ping message
    Ping(PingMessage),
    /// Pong message
    Pong(PongMessage),
    /// Close message
    Close(CloseMessage),
}

impl Message {
    /// Create a text message
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(TextMessage::new(text))
    }

    /// Create a binary message
    pub fn binary(data: impl Into<Bytes>) -> Self {
        Self::Binary(BinaryMessage::new(data))
    }

    /// Create a ping message
    pub fn ping(data: Option<Vec<u8>>) -> Self {
        Self::Ping(PingMessage::new(data))
    }

    /// Create a pong message
    pub fn pong(data: Option<Vec<u8>>) -> Self {
        Self::Pong(PongMessage::new(data))
    }

    /// Create a close message
    pub fn close(code: Option<u16>, reason: Option<String>) -> Self {
        Self::Close(CloseMessage::new(code, reason))
    }

    /// Get the message kind
    pub fn kind(&self) -> MessageKind {
        match self {
            Message::Text(_) => MessageKind::Text,
            Message::Binary(_) => MessageKind::Binary,
            Message::Ping(_) => MessageKind::Ping,
            Message::Pong(_) => MessageKind::Pong,
            Message::Close(_) => MessageKind::Close,
        }
    }

    /// Check if this is a control message
    pub fn is_control(&self) -> bool {
        matches!(self, Message::Ping(_) | Message::Pong(_) | Message::Close(_))
    }

    /// Check if this is a data message
    pub fn is_data(&self) -> bool {
        matches!(self, Message::Text(_) | Message::Binary(_))
    }

    /// Get the message payload as text
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Message::Text(msg) => Some(msg.as_str()),
            _ => None,
        }
    }

    /// Get the message payload as bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Message::Text(msg) => msg.as_bytes(),
            Message::Binary(msg) => msg.as_bytes(),
            Message::Ping(msg) => msg.as_bytes(),
            Message::Pong(msg) => msg.as_bytes(),
            Message::Close(msg) => msg.as_bytes(),
        }
    }

    /// Convert message to frames
    pub fn to_frames(&self) -> Vec<Frame> {
        match self {
            Message::Text(msg) => vec![msg.to_frame()],
            Message::Binary(msg) => vec![msg.to_frame()],
            Message::Ping(msg) => vec![msg.to_frame()],
            Message::Pong(msg) => vec![msg.to_frame()],
            Message::Close(msg) => vec![msg.to_frame()],
        }
    }

    /// Convert message to a single frame
    pub fn to_frame(&self) -> Frame {
        match self {
            Message::Text(msg) => msg.to_frame(),
            Message::Binary(msg) => msg.to_frame(),
            Message::Ping(msg) => msg.to_frame(),
            Message::Pong(msg) => msg.to_frame(),
            Message::Close(msg) => msg.to_frame(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Text(msg) => write!(f, "Text({})", msg.as_str()),
            Message::Binary(msg) => write!(f, "Binary({} bytes)", msg.len()),
            Message::Ping(msg) => write!(f, "Ping({} bytes)", msg.len()),
            Message::Pong(msg) => write!(f, "Pong({} bytes)", msg.len()),
            Message::Close(msg) => write!(f, "Close({:?})", msg),
        }
    }
}

/// Message kind for easier matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    /// Text message
    Text,
    /// Binary message
    Binary,
    /// Ping message
    Ping,
    /// Pong message
    Pong,
    /// Close message
    Close,
}

/// Text message
#[derive(Debug, Clone)]
pub struct TextMessage {
    text: String,
}

impl TextMessage {
    /// Create a new text message
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Get the text content
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Get the text as bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    /// Get the text length
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Check if the text is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Convert to frame
    pub fn to_frame(&self) -> Frame {
        Frame::text(self.text.clone())
    }
}

/// Binary message
#[derive(Debug, Clone)]
pub struct BinaryMessage {
    data: Bytes,
}

impl BinaryMessage {
    /// Create a new binary message
    pub fn new(data: impl Into<Bytes>) -> Self {
        Self { data: data.into() }
    }

    /// Get the binary data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to frame
    pub fn to_frame(&self) -> Frame {
        Frame::binary(self.data.clone())
    }
}

/// Ping message
#[derive(Debug, Clone)]
pub struct PingMessage {
    data: Bytes,
}

impl PingMessage {
    /// Create a new ping message
    pub fn new(data: Option<Vec<u8>>) -> Self {
        Self {
            data: data.map_or_else(Bytes::new, Bytes::from),
        }
    }

    /// Get the ping data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to frame
    pub fn to_frame(&self) -> Frame {
        Frame::ping(self.data.clone())
    }
}

/// Pong message
#[derive(Debug, Clone)]
pub struct PongMessage {
    data: Bytes,
}

impl PongMessage {
    /// Create a new pong message
    pub fn new(data: Option<Vec<u8>>) -> Self {
        Self {
            data: data.map_or_else(Bytes::new, Bytes::from),
        }
    }

    /// Get the pong data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to frame
    pub fn to_frame(&self) -> Frame {
        Frame::pong(self.data.clone())
    }
}

/// Close message
#[derive(Debug, Clone)]
pub struct CloseMessage {
    code: Option<u16>,
    reason: String,
}

impl CloseMessage {
    /// Create a new close message
    pub fn new(code: Option<u16>, reason: Option<String>) -> Self {
        Self {
            code,
            reason: reason.unwrap_or_default(),
        }
    }

    /// Get the close code
    pub fn code(&self) -> Option<u16> {
        self.code
    }

    /// Get the close reason
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Get the close code as CloseCode enum
    pub fn close_code(&self) -> Option<CloseCode> {
        self.code.map(CloseCode::from)
    }

    /// Get the message as bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.reason.as_bytes()
    }

    /// Get the total payload length
    pub fn len(&self) -> usize {
        let mut len = if self.code.is_some() { 2 } else { 0 };
        len += self.reason.len();
        len
    }

    /// Check if the message is empty
    pub fn is_empty(&self) -> bool {
        self.code.is_none() && self.reason.is_empty()
    }

    /// Convert to frame
    pub fn to_frame(&self) -> Frame {
        Frame::close(self.code, if self.reason.is_empty() { None } else { Some(&self.reason) })
    }
}

/// Message assembler for fragmented messages
#[derive(Debug, Default)]
pub struct MessageAssembler {
    /// Buffer for assembling fragmented messages
    buffer: BytesMut,
    /// Expected opcode for the message being assembled
    opcode: Option<Opcode>,
    /// Whether we're currently assembling a message
    assembling: bool,
}

impl MessageAssembler {
    /// Create a new message assembler
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a frame and try to assemble a complete message
    pub fn feed_frame(&mut self, frame: Frame) -> Result<Option<Message>> {
        if frame.is_control() {
            // Control frames are never fragmented
            return Ok(Some(self.control_frame_to_message(frame)?));
        }

        if !frame.fin {
            // Fragmented frame
            if !self.assembling {
                // Start of fragmented message
                self.assembling = true;
                self.opcode = Some(frame.opcode);
                self.buffer.extend_from_slice(&frame.payload);
                Ok(None)
            } else {
                // Continuation of fragmented message
                if frame.opcode != Opcode::Continuation {
                    return Err(Error::Protocol(ProtocolError::InvalidFrame(
                        "Expected continuation frame in fragmented message".to_string(),
                    )));
                }
                self.buffer.extend_from_slice(&frame.payload);
                Ok(None)
            }
        } else {
            // Final frame
            if self.assembling {
                // End of fragmented message
                self.buffer.extend_from_slice(&frame.payload);
                let message = self.assemble_complete_message()?;
                self.reset();
                Ok(Some(message))
            } else {
                // Single unfragmented frame
                self.opcode = Some(frame.opcode);
                self.buffer = BytesMut::from(&frame.payload[..]);
                let message = self.assemble_complete_message()?;
                self.reset();
                Ok(Some(message))
            }
        }
    }

    /// Convert a control frame to a message
    fn control_frame_to_message(&self, frame: Frame) -> Result<Message> {
        match frame.kind() {
            FrameKind::Ping => Ok(Message::ping(Some(frame.payload.to_vec()))),
            FrameKind::Pong => Ok(Message::pong(Some(frame.payload.to_vec()))),
            FrameKind::Close => {
                let (code, reason) = self.parse_close_payload(&frame.payload)?;
                Ok(Message::close(code, reason))
            }
            _ => Err(Error::Protocol(ProtocolError::InvalidFrame(
                "Unexpected control frame type".to_string(),
            ))),
        }
    }

    /// Parse close frame payload
    fn parse_close_payload(&self, payload: &[u8]) -> Result<(Option<u16>, Option<String>)> {
        if payload.len() < 2 {
            return Ok((None, None));
        }

        let code = u16::from_be_bytes([payload[0], payload[1]]);
        let reason = if payload.len() > 2 {
            String::from_utf8_lossy(&payload[2..]).to_string()
        } else {
            String::new()
        };

        Ok((Some(code), Some(reason)))
    }

    /// Assemble a complete message from the buffer
    fn assemble_complete_message(&self) -> Result<Message> {
        let opcode = self.opcode.ok_or_else(|| {
            Error::Protocol(ProtocolError::InvalidFrame("No opcode set".to_string()))
        })?;

        match opcode {
            Opcode::Text => {
                let text = String::from_utf8(self.buffer.to_vec())
                    .map_err(|_| Error::InvalidUtf8)?;
                Ok(Message::text(text))
            }
            Opcode::Binary => Ok(Message::binary(self.buffer.clone().freeze())),
            _ => Err(Error::Protocol(ProtocolError::InvalidFrame(
                "Unexpected opcode for data frame".to_string(),
            ))),
        }
    }

    /// Reset the assembler state
    fn reset(&mut self) {
        self.buffer.clear();
        self.opcode = None;
        self.assembling = false;
    }

    /// Check if currently assembling a message
    pub fn is_assembling(&self) -> bool {
        self.assembling
    }

    /// Get the number of bytes currently buffered
    pub fn buffered_bytes(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the assembler
    pub fn clear(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Opcode;

    #[test]
    fn test_text_message() {
        let msg = Message::text("hello");
        assert_eq!(msg.kind(), MessageKind::Text);
        assert_eq!(msg.as_text(), Some("hello"));
        assert!(msg.is_data());
        assert!(!msg.is_control());
    }

    #[test]
    fn test_binary_message() {
        let data = vec![1, 2, 3, 4];
        let msg = Message::binary(data.clone());
        assert_eq!(msg.kind(), MessageKind::Binary);
        assert_eq!(msg.as_bytes(), &data[..]);
        assert!(msg.is_data());
        assert!(!msg.is_control());
    }

    #[test]
    fn test_control_messages() {
        let ping = Message::ping(Some(vec![1, 2, 3]));
        let pong = Message::pong(Some(vec![4, 5, 6]));
        let close = Message::close(Some(1000), Some("Goodbye".to_string()));

        assert!(ping.is_control());
        assert!(pong.is_control());
        assert!(close.is_control());
    }

    #[test]
    fn test_close_message() {
        let msg = Message::close(Some(1000), Some("Goodbye".to_string()));
        if let Message::Close(close_msg) = msg {
            assert_eq!(close_msg.code(), Some(1000));
            assert_eq!(close_msg.reason(), "Goodbye");
        } else {
            panic!("Expected close message");
        }
    }

    #[test]
    fn test_message_assembler() {
        let mut assembler = MessageAssembler::new();

        // Feed fragmented text frames
        let frame1 = Frame::new(Opcode::Text, "Hello, ").fin(false);
        let frame2 = Frame::new(Opcode::Continuation, "world!").fin(true);

        let msg1 = assembler.feed_frame(frame1).unwrap();
        assert!(msg1.is_none()); // Not complete yet
        assert!(assembler.is_assembling());

        let msg2 = assembler.feed_frame(frame2).unwrap();
        assert!(msg2.is_some()); // Complete message
        assert!(!assembler.is_assembling());

        if let Some(Message::Text(text_msg)) = msg2 {
            assert_eq!(text_msg.as_str(), "Hello, world!");
        } else {
            panic!("Expected text message");
        }
    }

    #[test]
    fn test_message_display() {
        let text_msg = Message::text("hello");
        let binary_msg = Message::binary(vec![1, 2, 3]);
        let ping_msg = Message::ping(Some(vec![4, 5]));

        assert_eq!(text_msg.to_string(), "Text(hello)");
        assert_eq!(binary_msg.to_string(), "Binary(3 bytes)");
        assert_eq!(ping_msg.to_string(), "Ping(3 bytes)");
    }
}
