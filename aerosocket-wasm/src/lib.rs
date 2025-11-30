//! WebAssembly support for AeroSocket
//!
//! This module provides WebSocket client functionality for WebAssembly targets,
//! enabling AeroSocket to run in browsers and WASM environments.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    missing_docs,
    missing_debug_implementations,
    clippy::new_without_default,
    unused_variables,
    dead_code
)]
#![doc(html_root_url = "https://docs.rs/aerosocket-wasm/")]

/// WebSocket client for WebAssembly environments
pub struct WebSocketClient {
    #[cfg(feature = "wasm-bindgen")]
    ws: Option<web_sys::WebSocket>,
    #[cfg(feature = "wasm-bindgen")]
    url: String,
    #[cfg(not(feature = "wasm-bindgen"))]
    _private: (),
}

/// Configuration for WebSocket client
pub struct WebSocketConfig {
    #[cfg(feature = "wasm-bindgen")]
    protocols: Vec<String>,
    #[cfg(not(feature = "wasm-bindgen"))]
    _private: (),
}

// Placeholder implementation
impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(_url: String) -> Self {
        #[cfg(not(feature = "wasm-bindgen"))]
        {
            Self { _private: () }
        }
        #[cfg(feature = "wasm-bindgen")]
        {
            Self {
                ws: None,
                url: _url,
            }
        }
    }
}

impl WebSocketConfig {
    /// Create a new WebSocket configuration
    pub fn new() -> Self {
        #[cfg(not(feature = "wasm-bindgen"))]
        {
            Self { _private: () }
        }
        #[cfg(feature = "wasm-bindgen")]
        {
            Self {
                protocols: Vec::new(),
            }
        }
    }
}

// WASM-specific implementation when feature is enabled
#[cfg(feature = "wasm-bindgen")]
mod wasm_impl {
    use super::*;
    use aerosocket_core::Error as CoreError;
    use js_sys::Uint8Array;
    use wasm_bindgen::prelude::*;
    use web_sys::{MessageEvent, WebSocket};

    // Helper function to convert errors
    fn error_to_js(error: CoreError) -> JsValue {
        JsValue::from_str(&error.to_string())
    }

    impl WebSocketClient {
        pub fn new_wasm(url: String) -> Self {
            Self { ws: None, url }
        }

        pub async fn connect(&mut self) -> Result<(), JsValue> {
            let ws = WebSocket::new(&self.url)
                .map_err(|e| JsValue::from_str(&e.as_string().unwrap_or_default()))?;

            let _ws_clone = ws.clone();
            let onopen_closure = Closure::wrap(Box::new(move |_event: MessageEvent| {
                web_sys::console::log_1(&"WebSocket connected".into());
            }) as Box<dyn Fn(MessageEvent)>);

            ws.set_onopen(Some(onopen_closure.as_ref().unchecked_ref()));
            onopen_closure.forget();

            self.ws = Some(ws);
            Ok(())
        }

        pub fn send_text(&self, text: &str) -> Result<(), JsValue> {
            match &self.ws {
                Some(ws) => ws
                    .send_with_str(text)
                    .map_err(|e| JsValue::from_str(&e.as_string().unwrap_or_default())),
                None => Err(JsValue::from_str("WebSocket not connected")),
            }
        }

        pub fn send_binary(&self, data: &[u8]) -> Result<(), JsValue> {
            match &self.ws {
                Some(ws) => {
                    let array = Uint8Array::from(data);
                    ws.send_with_array_buffer(&array.buffer())
                        .map_err(|e| JsValue::from_str(&e.as_string().unwrap_or_default()))
                }
                None => Err(JsValue::from_str("WebSocket not connected")),
            }
        }

        pub fn close(&self) -> Result<(), JsValue> {
            match &self.ws {
                Some(ws) => ws
                    .close()
                    .map_err(|e| JsValue::from_str(&e.as_string().unwrap_or_default())),
                None => Err(JsValue::from_str("WebSocket not connected")),
            }
        }
    }

    impl WebSocketConfig {
        pub fn new_wasm() -> Self {
            Self {
                protocols: Vec::new(),
            }
        }

        pub fn with_protocol(mut self, protocol: String) -> Self {
            self.protocols.push(protocol);
            self
        }
    }
}

/// Prelude module
pub mod prelude {
    pub use crate::{WebSocketClient, WebSocketConfig};
    pub use aerosocket_core::prelude::*;
}
