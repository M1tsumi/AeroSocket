use std::{slice, str};

/// Transform the incoming message in-place in linear memory.
///
/// Host contract:
/// - `ptr` points to UTF-8 bytes of length `len`.
/// - We may overwrite up to `len` bytes starting at `ptr`.
/// - Return value is the number of valid UTF-8 bytes we wrote.
#[no_mangle]
pub extern "C" fn on_message(ptr: i32, len: i32) -> i32 {
    if ptr <= 0 || len <= 0 {
        return 0;
    }

    let ptr = ptr as usize;
    let len = len as usize;

    // Safety: host guarantees that `ptr..ptr+len` is valid writable memory.
    let buf = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, len) };

    // Interpret input as UTF-8 text (fallback to empty on error).
    let input = match str::from_utf8(buf) {
        Ok(s) => s,
        Err(_) => "",
    };

    // Simple transformation: prefix the message.
    let response = format!("WASM: {}", input);

    let bytes = response.as_bytes();
    let out_len = bytes.len().min(buf.len());

    // Write response back into the same region.
    buf[..out_len].copy_from_slice(&bytes[..out_len]);

    out_len as i32
}
