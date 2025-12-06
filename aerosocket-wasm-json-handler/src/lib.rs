use std::{slice, str};
use serde_json::{Value, json};

/// JSON-aware WASM handler.
///
/// Contract:
/// - Input: UTF-8 JSON text at `ptr` of length `len`.
/// - Output: UTF-8 JSON text written back starting at `ptr`.
/// - Return: number of bytes written (0 on error).
#[no_mangle]
pub extern "C" fn on_message(ptr: i32, len: i32) -> i32 {
    if ptr <= 0 || len <= 0 {
        return 0;
    }

    let ptr = ptr as usize;
    let len = len as usize;

    // Safety: host guarantees `ptr..ptr+len` is valid & writable.
    let buf = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, len) };

    // Interpret as UTF-8.
    let input_str = match str::from_utf8(buf) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // Parse JSON (generic Value so caller can send any shape).
    let parsed: Value = match serde_json::from_str(input_str) {
        Ok(v) => v,
        Err(_) => {
            // Echo an error envelope instead of 0.
            let err = json!({
                "error": "invalid_json",
                "original": input_str,
            });
            return write_response(buf, &err);
        }
    };

    // Example transformation: wrap original payload in an envelope with metadata.
    let response = json!({
        "handled_by": "aerosocket-wasm-json",
        "version": "0.1.0",
        "payload": parsed,
    });

    write_response(buf, &response)
}

/// Serialize JSON into the given buffer and return the number of bytes written.
fn write_response(buf: &mut [u8], value: &Value) -> i32 {
    let json_str = match serde_json::to_string(value) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let bytes = json_str.as_bytes();
    let out_len = bytes.len().min(buf.len());
    buf[..out_len].copy_from_slice(&bytes[..out_len]);
    out_len as i32
}
