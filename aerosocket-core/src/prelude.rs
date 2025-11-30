//! Prelude module for AeroSocket Core
//!
//! This module re-exports commonly used types and traits to make them
//! easily accessible for users of the library.

pub use crate::error::{CloseCode, Error, Result};
pub use crate::frame::{Frame, FrameKind};
pub use crate::message::{Message, MessageKind};
pub use crate::protocol::Opcode;
pub use crate::transport::Transport;

// Re-export commonly used external dependencies
pub use bytes::{Bytes, BytesMut};
pub use futures_util::Stream as FuturesStream;
pub use thiserror::Error as ThisError;

// Feature-gated re-exports
#[cfg(feature = "tokio-runtime")]
pub use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[cfg(feature = "async-std-runtime")]
pub use async_std::io::{Read as AsyncRead, Write as AsyncWrite};

#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};

#[cfg(feature = "rkyv")]
pub use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
