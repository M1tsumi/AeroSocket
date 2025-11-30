//! WebAssembly WebSocket client implementation
//!
//! This module provides WebSocket client functionality for WebAssembly targets,
//! enabling AeroSocket to run in browsers and WASM environments.

use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, ErrorEvent, CloseEvent, BinaryType, Blob};
use js_sys::Uint8Array;
use aerosocket_core::{Message, Result, Error};
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

/// WebAssembly-based WebSocket client
#[wasm_bindgen]
pub struct WebSocketClient {
    inner: web_sys::WebSocket,
    on_message_callback: Option<Closure<dyn FnMut(MessageEvent)>>,
    on_error_callback: Option<Closure<dyn FnMut(ErrorEvent)>>,
    on_close_callback: Option<Closure<dyn FnMut(CloseEvent)>>,
}

/// Configuration for WebSocket client
#[wasm_bindgen]
pub struct WebSocketConfig {
    url: String,
    protocols: Option<Vec<String>>,
}

#[wasm_bindgen]
impl WebSocketConfig {
    /// Create a new WebSocket configuration
    #[wasm_bindgen(constructor)]
    pub fn new(url: String) -> WebSocketConfig {
        WebSocketConfig {
            url,
            protocols: None,
        }
    }

    /// Set the sub-protocols to use
    #[wasm_bindgen(js_name = setProtocols)]
    pub fn set_protocols(&mut self, protocols: Vec<String>) {
        self.protocols = Some(protocols);
    }
}

#[wasm_bindgen]
impl WebSocketClient {
    /// Create a new WebSocket client
    #[wasm_bindgen(constructor)]
    pub fn new(config: WebSocketConfig) -> Result<WebSocketClient> {
        let ws = match &config.protocols {
            Some(protocols) => {
                let js_array = protocols
                    .iter()
                    .map(|p| JsValue::from_str(p))
                    .collect::<js_sys::Array>();
                web_sys::WebSocket::new_with_str_and_protocols(&config.url, &js_array)
            }
            None => web_sys::WebSocket::new(&config.url),
        }
        .map_err(|e| Error::Other(format!("Failed to create WebSocket: {:?}", e)))?;

        Ok(WebSocketClient {
            inner: ws,
            on_message_callback: None,
            on_error_callback: None,
            on_close_callback: None,
        })
    }

    /// Connect to the WebSocket server
    pub async fn connect(&mut self) -> Result<()> {
        // In a real implementation, this would handle the connection process
        // For now, we'll just return Ok as a placeholder
        Ok(())
    }

    /// Send a message
    pub fn send(&self, message: Message) -> Result<()> {
        match message {
            Message::Text(text) => {
                self.inner
                    .send_with_str(&text)
                    .map_err(|e| Error::Other(format!("Failed to send text message: {:?}", e)))?;
            }
            Message::Binary(data) => {
                let array = Uint8Array::new_with_length(data.len() as u32);
                for (i, byte) in data.iter().enumerate() {
                    array.set_index(i as u32, *byte);
                }
                self.inner.send_with_array_buffer(&array.buffer())
                    .map_err(|e| Error::Other(format!("Failed to send binary message: {:?}", e)))?;
            }
            Message::Ping(_) => {
                // Ping messages are handled internally by the browser
                return Ok(());
            }
            Message::Pong(_) => {
                // Pong messages are handled internally by the browser
                return Ok(());
            }
            Message::Close(code, reason) => {
                self.inner
                    .close_with_code_and_reason(code.into(), &reason)
                    .map_err(|e| Error::Other(format!("Failed to send close message: {:?}", e)))?;
            }
        }
        Ok(())
    }

    /// Set message callback
    #[wasm_bindgen(js_name = onMessage)]
    pub fn on_message(&mut self, callback: &js_sys::Function) {
        let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            let _ = callback.call1(&JsValue::NULL, &event);
        }) as Box<dyn FnMut(MessageEvent)>);

        self.inner
            .set_onmessage(Some(callback.as_ref().unchecked_ref()));
        self.on_message_callback = Some(callback);
    }

    /// Set error callback
    #[wasm_bindgen(js_name = onError)]
    pub fn on_error(&mut self, callback: &js_sys::Function) {
        let callback = Closure::wrap(Box::new(move |event: ErrorEvent| {
            let _ = callback.call1(&JsValue::NULL, &event);
        }) as Box<dyn FnMut(ErrorEvent)>);

        self.inner
            .set_onerror(Some(callback.as_ref().unchecked_ref()));
        self.on_error_callback = Some(callback);
    }

    /// Set close callback
    #[wasm_bindgen(js_name = onClose)]
    pub fn on_close(&mut self, callback: &js_sys::Function) {
        let callback = Closure::wrap(Box::new(move |event: CloseEvent| {
            let _ = callback.call1(&JsValue::NULL, &event);
        }) as Box<dyn FnMut(CloseEvent)>);

        self.inner
            .set_onclose(Some(callback.as_ref().unchecked_ref()));
        self.on_close_callback = Some(callback);
    }

    /// Get the current ready state
    #[wasm_bindgen(getter)]
    pub fn ready_state(&self) -> u16 {
        self.inner.ready_state()
    }

    /// Check if the connection is open
    #[wasm_bindgen(js_name = isOpen)]
    pub fn is_open(&self) -> bool {
        self.inner.ready_state() == web_sys::WebSocket::OPEN
    }
}

impl Drop for WebSocketClient {
    fn drop(&mut self) {
        // Clean up callbacks
        self.on_message_callback = None;
        self.on_error_callback = None;
        self.on_close_callback = None;

        // Close the WebSocket if it's still open
        if self.inner.ready_state() == web_sys::WebSocket::OPEN {
            let _ = self.inner.close();
        }
    }
}
