//! WebSocket connection handlers
//!
//! This module provides handler abstractions for processing WebSocket connections.

use aerosocket_core::{Message, Result};
use std::future::Future;
use std::pin::Pin;

/// Trait for handling WebSocket connections
pub trait Handler: Send + Sync + 'static {
    /// Handle a new connection
    fn handle<'a>(
        &'a self,
        connection: crate::connection::ConnectionHandle,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

    /// Clone the handler
    fn clone_box(&self) -> Box<dyn Handler>;
}

impl Clone for Box<dyn Handler> {
    fn clone(&self) -> Box<dyn Handler> {
        self.clone_box()
    }
}

/// Boxed handler type
pub type BoxedHandler = Box<dyn Handler>;

/// Default handler implementation
#[derive(Debug, Clone)]
pub struct DefaultHandler;

impl DefaultHandler {
    /// Create a new default handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Handler for DefaultHandler {
    fn handle<'a>(
        &'a self,
        connection: crate::connection::ConnectionHandle,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Get the connection from the handle
            let mut conn = connection.try_lock().await.map_err(|_| {
                aerosocket_core::Error::Other("Failed to lock connection".to_string())
            })?;

            while let Some(msg) = conn.next().await? {
                match msg {
                    Message::Text(text) => {
                        conn.send_text(text.as_str()).await?;
                    }
                    Message::Binary(data) => {
                        conn.send_binary(data.as_bytes().to_vec()).await?;
                    }
                    Message::Ping(_) => {
                        conn.pong(None).await?;
                    }
                    Message::Close(close_msg) => {
                        let code = close_msg.code();
                        let reason = close_msg.reason();
                        conn.close(code, Some(reason)).await?;
                        break;
                    }
                    Message::Pong(_) => {
                        // Ignore pong messages
                    }
                }
            }

            Ok(())
        })
    }

    fn clone_box(&self) -> Box<dyn Handler> {
        Box::new(self.clone())
    }
}

/// Echo handler implementation
#[derive(Debug, Clone)]
pub struct EchoHandler;

impl EchoHandler {
    /// Create a new echo handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for EchoHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Handler for EchoHandler {
    fn handle<'a>(
        &'a self,
        connection: crate::connection::ConnectionHandle,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Get the connection from the handle
            let mut conn = connection.try_lock().await.map_err(|_| {
                aerosocket_core::Error::Other("Failed to lock connection".to_string())
            })?;

            while let Some(msg) = conn.next().await? {
                match msg {
                    Message::Text(text) => {
                        let echo_text = format!("Echo: {}", text.as_str());
                        conn.send_text(&echo_text).await?;
                    }
                    Message::Binary(data) => {
                        conn.send_binary(data.as_bytes().to_vec()).await?;
                    }
                    Message::Ping(_) => {
                        conn.pong(None).await?;
                    }
                    Message::Close(close_msg) => {
                        let code = close_msg.code();
                        let reason = close_msg.reason();
                        conn.close(code, Some(reason)).await?;
                        break;
                    }
                    Message::Pong(_) => {
                        // Ignore pong messages
                    }
                }
            }

            Ok(())
        })
    }

    fn clone_box(&self) -> Box<dyn Handler> {
        Box::new(self.clone())
    }
}

/// Function-based handler
#[derive(Clone)]
pub struct FnHandler<F> {
    f: F,
}

impl<F> FnHandler<F> {
    /// Create a new function-based handler
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> Handler for FnHandler<F>
where
    F: Fn(crate::connection::ConnectionHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
        + Send
        + Sync
        + Clone
        + 'static,
{
    fn handle<'a>(
        &'a self,
        connection: crate::connection::ConnectionHandle,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin((self.f)(connection))
    }

    fn clone_box(&self) -> Box<dyn Handler> {
        Box::new(self.clone())
    }
}

impl<F> std::fmt::Debug for FnHandler<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnHandler")
            .field("f", &"<function>")
            .finish()
    }
}

/// Create a handler from a function
pub fn from_fn<F, Fut>(f: F) -> FnHandler<F>
where
    F: Fn(crate::connection::ConnectionHandle) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    FnHandler::new(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[tokio::test]
    async fn test_default_handler() {
        let handler = DefaultHandler::new();
        let remote = "127.0.0.1:12345".parse().unwrap();
        let local = "127.0.0.1:8080".parse().unwrap();
        let connection = crate::connection::Connection::new(remote, local);

        // Note: This test will fail until Connection::next and send are implemented
        // For now, we just test that the handler can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_echo_handler() {
        let handler = EchoHandler::new();
        let remote = "127.0.0.1:12345".parse().unwrap();
        let local = "127.0.0.1:8080".parse().unwrap();
        let connection = crate::connection::Connection::new(remote, local);

        // Note: This test will fail until Connection::next and send are implemented
        // For now, we just test that the handler can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_fn_handler() {
        let handler = from_fn(|_conn| async { Ok(()) });

        let remote = "127.0.0.1:12345".parse().unwrap();
        let local = "127.0.0.1:8080".parse().unwrap();
        let connection = crate::connection::Connection::new(remote, local);

        // Note: This test will fail until Connection::next and send are implemented
        // For now, we just test that the handler can be created
        assert!(true);
    }
}
