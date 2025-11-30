// WebAssembly WebSocket client implementation

// Always export the types
pub struct WebSocketClient {
    #[cfg(feature = "wasm-bindgen")]
    ws: Option<web_sys::WebSocket>,
    #[cfg(feature = "wasm-bindgen")]
    url: String,
    #[cfg(not(feature = "wasm-bindgen"))]
    _private: (),
}

pub struct WebSocketConfig {
    #[cfg(feature = "wasm-bindgen")]
    protocols: Vec<String>,
    #[cfg(not(feature = "wasm-bindgen"))]
    _private: (),
}

// Placeholder implementation
impl WebSocketClient {
    pub fn new(_url: String) -> Self {
        Self { _private: () }
    }
}

impl WebSocketConfig {
    pub fn new() -> Self {
        Self { _private: () }
    }
}