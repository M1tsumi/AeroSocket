//! WebSocket frame parsing and serialization
//!
//! This module provides efficient zero-copy frame parsing and serialization
//! following the RFC 6455 WebSocket protocol specification.

use crate::{
    error::{Error, FrameError, Result},
    protocol::{frame::*, Opcode},
};
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Represents a WebSocket frame according to RFC 6455
#[derive(Debug, Clone)]
pub struct Frame {
    /// Indicates if this is the final frame in a message
    pub fin: bool,
    /// Reserved bits (RSV1, RSV2, RSV3)
    pub rsv: [bool; 3],
    /// Frame opcode
    pub opcode: Opcode,
    /// Indicates if the payload is masked
    pub masked: bool,
    /// Masking key (if present)
    pub mask: Option<[u8; 4]>,
    /// Payload data
    pub payload: Bytes,
}

impl Frame {
    /// Create a new frame with the given opcode and payload
    pub fn new(opcode: Opcode, payload: impl Into<Bytes>) -> Self {
        Self {
            fin: true,
            rsv: [false; 3],
            opcode,
            masked: false,
            mask: None,
            payload: payload.into(),
        }
    }

    /// Create a continuation frame
    pub fn continuation(payload: impl Into<Bytes>) -> Self {
        Self::new(Opcode::Continuation, payload)
    }

    /// Create a text frame
    pub fn text(payload: impl Into<Bytes>) -> Self {
        Self::new(Opcode::Text, payload)
    }

    /// Create a binary frame
    pub fn binary(payload: impl Into<Bytes>) -> Self {
        Self::new(Opcode::Binary, payload)
    }

    /// Create a close frame with optional code and reason
    pub fn close(code: Option<u16>, reason: Option<&str>) -> Self {
        let mut payload = BytesMut::new();

        if let Some(code) = code {
            payload.put_u16(code);
        }

        if let Some(reason) = reason {
            payload.put_slice(reason.as_bytes());
        }

        Self::new(Opcode::Close, payload.freeze())
    }

    /// Create a ping frame
    pub fn ping(payload: impl Into<Bytes>) -> Self {
        Self::new(Opcode::Ping, payload)
    }

    /// Create a pong frame
    pub fn pong(payload: impl Into<Bytes>) -> Self {
        Self::new(Opcode::Pong, payload)
    }

    /// Set the FIN bit
    pub fn fin(mut self, fin: bool) -> Self {
        self.fin = fin;
        self
    }

    /// Set reserved bits
    pub fn rsv(mut self, rsv1: bool, rsv2: bool, rsv3: bool) -> Self {
        self.rsv = [rsv1, rsv2, rsv3];
        self
    }

    /// Apply masking to the frame (for client frames)
    pub fn mask(mut self, enabled: bool) -> Self {
        if enabled && !self.masked {
            let mask = rand::random::<[u8; 4]>();
            self.payload = mask_bytes(&self.payload, &mask);
            self.masked = true;
            self.mask = Some(mask);
        } else if !enabled && self.masked {
            // Unmask if possible (server frames should never be masked)
            if let Some(mask) = self.mask {
                self.payload = mask_bytes(&self.payload, &mask);
            }
            self.masked = false;
            self.mask = None;
        }
        self
    }

    /// Apply compression to the frame (for data frames)
    #[cfg(feature = "compression")]
    pub fn compress(mut self, enabled: bool) -> Self {
        if enabled && self.opcode.is_data() && !self.rsv[0] {
            use flate2::write::DeflateEncoder;
            use flate2::Compression;
            use std::io::Write;

            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(6));
            if encoder.write_all(&self.payload).is_ok() && encoder.flush().is_ok() {
                if let Ok(compressed) = encoder.finish() {
                    self.payload = Bytes::from(compressed);
                    self.rsv[0] = true;
                }
            }
        }
        self
    }

    /// Serialize the frame to bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::new();
        self.write_to(&mut buf);
        buf.freeze()
    }

    /// Write the frame to a buffer
    pub fn write_to(&self, buf: &mut BytesMut) {
        // Write first byte
        let first_byte = ((self.fin as u8) << 7)
            | ((self.rsv[0] as u8) << 6)
            | ((self.rsv[1] as u8) << 5)
            | ((self.rsv[2] as u8) << 4)
            | self.opcode.value();
        buf.put_u8(first_byte);

        // Write payload length and mask bit
        let payload_len = self.payload.len();
        let mask_bit = (self.masked as u8) << 7;

        if payload_len < 126 {
            buf.put_u8(mask_bit | payload_len as u8);
        } else if payload_len <= u16::MAX as usize {
            buf.put_u8(mask_bit | PAYLOAD_LEN_16);
            buf.put_u16(payload_len as u16);
        } else {
            buf.put_u8(mask_bit | PAYLOAD_LEN_64);
            buf.put_u64(payload_len as u64);
        }

        // Write masking key if present
        if let Some(mask) = self.mask {
            buf.put_slice(&mask);
        }

        // Write payload
        buf.put_slice(&self.payload);
    }

    /// Parse a frame from bytes
    pub fn parse(buf: &mut BytesMut, compression_enabled: bool) -> Result<Self> {
        if buf.len() < 2 {
            return Err(FrameError::InsufficientData {
                needed: 2,
                have: buf.len(),
            }
            .into());
        }

        let mut cursor = std::io::Cursor::new(&buf[..]);

        // Read first byte
        let first_byte = cursor.get_u8();
        let fin = (first_byte & FIN_BIT) != 0;
        let rsv1 = (first_byte & RSV1_BIT) != 0;
        let rsv2 = (first_byte & RSV2_BIT) != 0;
        let rsv3 = (first_byte & RSV3_BIT) != 0;
        let opcode = Opcode::from(first_byte & OPCODE_MASK)
            .ok_or(FrameError::InvalidOpcode(first_byte & OPCODE_MASK))?;

        // Read second byte
        let second_byte = cursor.get_u8();
        let masked = (second_byte & MASK_BIT) != 0;
        let mut payload_len = (second_byte & PAYLOAD_LEN_MASK) as usize;

        // Read extended payload length if needed
        if payload_len == 126 {
            if buf.len() < 4 {
                return Err(FrameError::InsufficientData {
                    needed: 4,
                    have: buf.len(),
                }
                .into());
            }
            payload_len = cursor.get_u16() as usize;
        } else if payload_len == 127 {
            if buf.len() < 10 {
                return Err(FrameError::InsufficientData {
                    needed: 10,
                    have: buf.len(),
                }
                .into());
            }
            payload_len = cursor.get_u64() as usize;
        }

        // Read masking key if present
        let mask = if masked {
            if buf.len() < cursor.position() as usize + 4 + payload_len {
                return Err(FrameError::InsufficientData {
                    needed: cursor.position() as usize + 4 + payload_len,
                    have: buf.len(),
                }
                .into());
            }
            let mut mask = [0u8; 4];
            cursor.copy_to_slice(&mut mask);
            Some(mask)
        } else {
            None
        };

        // Read payload
        if buf.len() < cursor.position() as usize + payload_len {
            return Err(FrameError::InsufficientData {
                needed: cursor.position() as usize + payload_len,
                have: buf.len(),
            }
            .into());
        }

        let mut payload = Bytes::copy_from_slice(
            &buf[cursor.position() as usize..cursor.position() as usize + payload_len],
        );

        // Unmask payload if needed
        if let Some(mask) = mask {
            payload = mask_bytes(&payload, &mask);
        }

        // Decompress payload if needed
        #[cfg(feature = "compression")]
        if rsv1 && compression_enabled {
            use flate2::read::DeflateDecoder;
            use std::io::Read;

            let mut decoder = DeflateDecoder::new(&payload[..]);
            let mut decompressed = Vec::new();
            if let Err(_) = decoder.read_to_end(&mut decompressed) {
                return Err(FrameError::DecompressionFailed.into());
            }
            payload = Bytes::from(decompressed);
        }

        // Advance the buffer
        let frame_len = cursor.position() as usize + payload_len;
        buf.advance(frame_len);

        // Validate frame
        if opcode.is_control() && !fin {
            return Err(FrameError::FragmentedControlFrame.into());
        }

        if (rsv1 && !(compression_enabled && opcode.is_data())) || rsv2 || rsv3 {
            return Err(FrameError::ReservedBitsSet.into());
        }

        Ok(Frame {
            fin,
            rsv: [rsv1, rsv2, rsv3],
            opcode,
            masked,
            mask,
            payload,
        })
    }

    /// Get the frame kind
    pub fn kind(&self) -> FrameKind {
        match self.opcode {
            Opcode::Text => FrameKind::Text,
            Opcode::Binary => FrameKind::Binary,
            Opcode::Close => FrameKind::Close,
            Opcode::Ping => FrameKind::Ping,
            Opcode::Pong => FrameKind::Pong,
            Opcode::Continuation => FrameKind::Continuation,
            _ => FrameKind::Reserved,
        }
    }

    /// Get the payload length
    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }

    /// Check if this is a control frame
    pub fn is_control(&self) -> bool {
        self.opcode.is_control()
    }

    /// Check if this is a data frame
    pub fn is_data(&self) -> bool {
        self.opcode.is_data()
    }

    /// Check if this is the final frame
    pub fn is_final(&self) -> bool {
        self.fin
    }
}

/// Frame kind for easier matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    /// Text frame
    Text,
    /// Binary frame
    Binary,
    /// Close frame
    Close,
    /// Ping frame
    Ping,
    /// Pong frame
    Pong,
    /// Continuation frame
    Continuation,
    /// Reserved frame
    Reserved,
}

/// Apply masking to bytes
fn mask_bytes(data: &[u8], mask: &[u8; 4]) -> Bytes {
    let mut masked = BytesMut::with_capacity(data.len());
    for (i, &byte) in data.iter().enumerate() {
        masked.put_u8(byte ^ mask[i % 4]);
    }
    masked.freeze()
}

/// Frame parser for incremental parsing
#[derive(Debug)]
pub struct FrameParser {
    /// Buffer for partial frame data
    buffer: BytesMut,
    /// Expected frame size (if known)
    expected_size: Option<usize>,
    /// Whether compression is enabled for this connection
    compression_enabled: bool,
}

impl Default for FrameParser {
    fn default() -> Self {
        Self {
            buffer: BytesMut::new(),
            expected_size: None,
            compression_enabled: false,
        }
    }
}

impl FrameParser {
    /// Create a new frame parser
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new frame parser with compression enabled
    pub fn with_compression(compression_enabled: bool) -> Self {
        Self {
            buffer: BytesMut::new(),
            expected_size: None,
            compression_enabled,
        }
    }

    /// Feed data to the parser and try to extract frames
    pub fn feed(&mut self, data: &[u8]) -> Vec<Result<Frame>> {
        self.buffer.extend_from_slice(data);
        self.extract_frames()
    }

    /// Extract complete frames from the buffer
    fn extract_frames(&mut self) -> Vec<Result<Frame>> {
        let mut frames = Vec::new();

        while let Some(frame) = self.try_parse_frame() {
            match frame {
                Ok(f) => frames.push(Ok(f)),
                Err(e) => {
                    frames.push(Err(e));
                    break;
                }
            }
        }

        frames
    }

    /// Try to parse a single frame from the buffer
    fn try_parse_frame(&mut self) -> Option<Result<Frame>> {
        let mut buf = self.buffer.clone();

        match Frame::parse(&mut buf, self.compression_enabled) {
            Ok(frame) => {
                // Remove the parsed data from the buffer
                let parsed_len = self.buffer.len() - buf.len();
                self.buffer.advance(parsed_len);
                Some(Ok(frame))
            }
            Err(Error::Frame(FrameError::InsufficientData { .. })) => {
                // Not enough data, wait for more
                None
            }
            Err(e) => {
                // Parse error, consume the problematic data
                self.buffer.clear();
                Some(Err(e))
            }
        }
    }

    /// Get the number of bytes currently buffered
    pub fn buffered_bytes(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the parser buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.expected_size = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_frame_serialization() {
        let frame = Frame::text("hello");
        let bytes = frame.to_bytes();

        assert_eq!(bytes[0], 0x81); // FIN=1, RSV=000, Opcode=0001
        assert_eq!(bytes[1], 0x05); // MASK=0, Length=5
        assert_eq!(&bytes[2..], b"hello");
    }

    #[test]
    fn test_masked_frame() {
        let frame = Frame::text("hello").mask(true);
        let bytes = frame.to_bytes();

        assert_eq!(bytes[1] & 0x80, 0x80); // MASK bit set
        assert_eq!(bytes.len(), 2 + 4 + 5); // header + mask + payload
    }

    #[test]
    fn test_frame_parsing() {
        let original = Frame::text("hello");
        let bytes = original.to_bytes();
        let mut buf = BytesMut::from(&bytes[..]);

        let parsed = Frame::parse(&mut buf, false).unwrap();
        assert_eq!(parsed.kind(), FrameKind::Text);
        assert_eq!(parsed.payload, "hello");
        assert!(buf.is_empty());
    }

    #[test]
    fn test_large_frame() {
        let payload = vec![0u8; 65536]; // 64KB
        let frame = Frame::binary(payload.clone());
        let bytes = frame.to_bytes();

        assert_eq!(bytes[1], 127); // Extended 64-bit length
        assert_eq!(bytes[2..10], (65536u64).to_be_bytes());
    }

    #[test]
    fn test_close_frame() {
        let frame = Frame::close(Some(1000), Some("Goodbye"));
        let bytes = frame.to_bytes();

        assert_eq!(bytes[0], 0x88); // FIN=1, Opcode=8
        assert_eq!(bytes[1], 0x09); // Length=9 (2 bytes code + 6 bytes reason + 1 byte for length prefix?)
        assert_eq!(&bytes[2..4], 1000u16.to_be_bytes());
        assert_eq!(&bytes[4..], b"Goodbye");
        assert_eq!(bytes.len(), 11); // Total frame length
    }

    #[test]
    fn test_frame_parser() {
        let mut parser = FrameParser::new();

        let frame1 = Frame::text("frame1");
        let frame2 = Frame::ping("ping");

        let bytes1 = frame1.to_bytes();
        let bytes2 = frame2.to_bytes();

        // Feed partial data
        let frames = parser.feed(&bytes1[..5]);
        assert_eq!(frames.len(), 0); // Not enough data

        // Feed remaining data
        let frames = parser.feed(&bytes1[5..]);
        assert_eq!(frames.len(), 1);
        assert!(frames[0].as_ref().unwrap().is_data());

        // Feed second frame
        let frames = parser.feed(&bytes2);
        assert_eq!(frames.len(), 1);
        assert!(frames[0].as_ref().unwrap().is_control());
    }
}
