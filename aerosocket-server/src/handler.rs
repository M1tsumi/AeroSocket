//! WebSocket connection handlers
//!
//! This module provides handler abstractions for processing WebSocket connections.

use aerosocket_core::{Message, Result};
use std::future::Future;
use std::pin::Pin;
#[cfg(feature = "wasm-handlers")]
use std::path::PathBuf;
#[cfg(feature = "wasm-handlers")]
use wasmtime::{Engine, Instance, Module, Store};

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

#[cfg(feature = "wasm-handlers")]
#[derive(Clone)]
pub struct WasmHandler {
    engine: Engine,
    module: Module,
    module_path: PathBuf,
}

#[cfg(feature = "wasm-handlers")]
impl WasmHandler {
    pub fn from_file(path: impl Into<PathBuf>) -> aerosocket_core::Result<Self> {
        let path_buf = path.into();
        let engine = Engine::default();
        let module = Module::from_file(&engine, &path_buf).map_err(|e| {
            aerosocket_core::Error::Other(format!("Failed to load WASM module: {}", e))
        })?;

        Ok(Self {
            engine,
            module,
            module_path: path_buf,
        })
    }
}

#[cfg(feature = "wasm-handlers")]
impl Handler for WasmHandler {
    fn handle<'a>(
        &'a self,
        connection: crate::connection::ConnectionHandle,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut conn = connection.try_lock().await.map_err(|_| {
                aerosocket_core::Error::Other("Failed to lock connection".to_string())
            })?;

            let mut store = Store::new(&self.engine, ());
            let instance = Instance::new(&mut store, &self.module, &[]).map_err(|e| {
                aerosocket_core::Error::Other(format!("Failed to instantiate WASM module: {}", e))
            })?;

            let memory = instance
                .get_memory(&mut store, "memory")
                .ok_or_else(|| aerosocket_core::Error::Other("WASM module missing `memory` export".to_string()))?;

            let func = instance
                .get_typed_func::<(i32, i32), i32>(&mut store, "on_message")
                .map_err(|e| {
                    aerosocket_core::Error::Other(format!(
                        "Failed to get WASM function `on_message`: {}",
                        e
                    ))
                })?;

            let capacity = memory.data_size(&store) as usize;

            loop {
                if let Some(msg) = conn.next().await? {
                    match msg {
                        Message::Text(text) => {
                            let bytes = text.as_bytes();
                            if bytes.len() > capacity {
                                return Err(aerosocket_core::Error::Other(
                                    "WASM memory too small for incoming message".to_string(),
                                ));
                            }

                            memory
                                .write(&mut store, 0, bytes)
                                .map_err(|e| {
                                    aerosocket_core::Error::Other(format!(
                                        "Failed to write to WASM memory: {}",
                                        e
                                    ))
                                })?;

                            let out_len = func
                                .call(&mut store, (0, bytes.len() as i32))
                                .map_err(|e| {
                                    aerosocket_core::Error::Other(format!(
                                        "WASM `on_message` call failed: {}",
                                        e
                                    ))
                                })?;

                            if out_len > 0 {
                                let mut out = vec![0u8; out_len as usize];
                                memory
                                    .read(&mut store, 0, &mut out)
                                    .map_err(|e| {
                                        aerosocket_core::Error::Other(format!(
                                            "Failed to read from WASM memory: {}",
                                            e
                                        ))
                                    })?;

                                let out_text = String::from_utf8_lossy(&out).to_string();
                                conn.send_text(&out_text).await?;
                            }
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
                        Message::Pong(_) => {}
                    }
                } else {
                    break;
                }
            }

            Ok(())
        })
    }

    fn clone_box(&self) -> Box<dyn Handler> {
        Box::new(self.clone())
    }
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
    }

    #[tokio::test]
    async fn test_echo_handler() {
        let handler = EchoHandler::new();
        let remote = "127.0.0.1:12345".parse().unwrap();
        let local = "127.0.0.1:8080".parse().unwrap();
        let connection = crate::connection::Connection::new(remote, local);

        // Note: This test will fail until Connection::next and send are implemented
        // For now, we just test that the handler can be created
    }

    #[tokio::test]
    async fn test_fn_handler() {
        let handler = from_fn(|_conn| async { Ok(()) });

        let remote = "127.0.0.1:12345".parse().unwrap();
        let local = "127.0.0.1:8080".parse().unwrap();
        let connection = crate::connection::Connection::new(remote, local);

        // Note: This test will fail until Connection::next and send are implemented
        // For now, we just test that the handler can be created
    }
}
