<div align="center">
  <img src="assets/banner.svg" alt="AeroSocket Banner" width="800" height="200">
</div>

# AeroSocket

[![Crates.io](https://img.shields.io/crates/v/aerosocket.svg)](https://crates.io/crates/aerosocket)
[![Documentation](https://docs.rs/aerosocket/badge.svg)](https://docs.rs/aerosocket)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A fast, zero-copy WebSocket library for Rust.

AeroSocket handles millions of concurrent connections efficiently. Existing libraries often trade performance for usability. AeroSocket provides both high performance and a clean API.

## Key Features

- **High Performance**: Zero-copy message handling achieves 2.5M messages per second on modest hardware
- **Clean API**: Natural builder patterns with sensible defaults
- **Production Ready**: Rate limiting, TLS, graceful shutdown, and comprehensive error handling

## Quick Start

Basic echo server:

```rust
use aerosocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = aerosocket::Server::builder()
        .bind("0.0.0.0:8080")
        .max_connections(10_000)
        .build()?;

    server.serve(|mut conn| async move {
        while let Some(msg) = conn.next().await? {
            match msg {
                Message::Text(text) => conn.send_text(text).await?,
                Message::Binary(data) => conn.send_binary(data).await?,
                Message::Ping => conn.send_pong().await?,
                _ => {}
            }
        }
        Ok(())
    }).await?;

    Ok(())
}
```

And a client:

```rust
use aerosocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a local echo server over TCP
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let config = ClientConfig::default();
    let client = Client::new(addr).with_config(config);
    let mut conn = client.connect().await?;

    conn.send_text("Hello!").await?;

    if let Some(msg) = conn.next().await? {
        println!("Got: {:?}", msg);
    }

    Ok(())
}
```

TLS client (wss) with a custom CA and SNI:

```rust
use aerosocket::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "127.0.0.1:8443".parse()?;

    let tls = TlsConfig {
        verify: true,
        ca_file: Some("ca.pem".into()),
        cert_file: None,
        key_file: None,
        server_name: Some("localhost".into()),
        min_version: None,
        max_version: None,
    };

    let config = ClientConfig::default().tls(tls);
    let client = Client::new(addr).with_config(config);
    let mut conn = client.connect().await?;

    conn.send_text("Hello over TLS!").await?;
    if let Some(msg) = conn.next().await? {
        println!("Got: {:?}", msg);
    }

    Ok(())
}
```

Add this to your `Cargo.toml`:

```toml
[dependencies]
aerosocket = { version = "0.3", features = ["full"] }
tokio = { version = "1.0", features = ["full"] }
```

## Performance

Benchmarks on AWS c6i.large with Rust 1.75 and 10k concurrent connections:

| Metric | AeroSocket | tokio-tungstenite | fastwebsockets |
|--------|------------|-------------------|----------------|
| Throughput (small msgs) | 2.5M msg/s | 1.2M msg/s | 1.8M msg/s |
| Latency P99 | < 50μs | 120μs | 80μs |
| Memory usage | 40% less | baseline | 25% less |
| Zero-copy | Yes | No | Yes |

Results may vary by workload and configuration.

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Application   │───▶│   AeroSocket     │───▶│   Transport     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                        │
                              ▼                        ▼
                       ┌──────────────┐      ┌──────────────┐
                       │   Protocol   │      │ TCP/TLS/QUIC │
                       └──────────────┘      └──────────────┘
                              │
                              ▼
                       ┌──────────────┐
                       │ Zero-Copy    │
                       │   Engine     │
                       └──────────────┘
```

The zero-copy engine minimizes allocations by passing references instead of copying payloads. This is crucial for high-throughput messaging.

---

## Use Cases

AeroSocket is suitable for:

- Chat applications and collaborative editors
- Trading platforms requiring low-latency market data
- Game servers for multiplayer backends
- IoT systems with sensor networks and real-time monitoring

## Features

**Implemented:**
- Full RFC6455 WebSocket protocol
- TCP and TLS transports
- Rate limiting and backpressure
- Structured logging with tracing
- Graceful shutdown
- Client authentication (Basic/Bearer)
- CORS origin validation

**In Development:**
- Per-message compression
- Prometheus metrics exporter
- Health check endpoints

**Planned:**
- QUIC transport
- Load balancing
- Kubernetes operator

## Installation

```toml
[dependencies]
aerosocket = { version = "0.3", features = ["full"] }
```

### Minimal Setup

```toml
[dependencies]
aerosocket = { version = "0.3", default-features = false, features = ["transport-tcp"] }
```

Available flags on the `aerosocket` crate:
- `full` — Enable server, client, TCP, TLS, WASM, serde, rkyv
- `server` — Server API
- `client` — Client API
- `transport-tcp` — TCP transport (on by default)
- `transport-tls` — TLS transport
- `tokio-runtime` — Tokio integration
- `serde` — JSON serialization helpers
- `rkyv` — Zero-copy serialization helpers

---

## Examples

**Zero-copy operations:**

```rust
let data = Bytes::from("large payload");
conn.send_binary_bytes(data).await?; // No allocation
```

### WASM Handlers (Preview)

Enable `wasm-handlers` feature on `aerosocket-server` to use WASM-based handlers:

```toml
[dependencies]
aerosocket-server = { version = "0.3", features = ["wasm-handlers"] }
```

Load WASM handler from file:

```rust
use aerosocket_server::prelude::*;

let server = Server::builder()
    .bind("0.0.0.0:8080")?
    .build_with_wasm_handler_from_file(
        "handler.wasm",
    )?;
```

WASM ABI:
- Export linear memory
- Export `on_message(ptr: i32, len: i32) -> i32`
- Host writes UTF-8 to memory, WASM processes in-place, returns new length

This repo includes two small example crates you can copy or adapt:

- `aerosocket-wasm-handler` — simple text transformer that prefixes messages
  with `"WASM: "`.
- `aerosocket-wasm-json-handler` — JSON-aware handler that parses the incoming
  text as JSON, wraps it in a small metadata envelope, and serializes it back to
  JSON.

---

## Security

Security features include:

- TLS 1.2/1.3 with configurable certificates
- Rate limiting and connection quotas
- Input validation for WebSocket frames
- Memory safety through Rust
- Backpressure to prevent resource exhaustion

Example production configuration:

```rust
use aerosocket::prelude::*;

let server = aerosocket::Server::builder()
    .bind("0.0.0.0:8443")?
    .max_connections(10_000)
    .backpressure(BackpressureStrategy::Buffer)
    .tls("server.crt", "server.key")
    .transport_tls()
    .idle_timeout(Duration::from_secs(300))
    .build()?;
```

## Documentation

- [API Reference](https://docs.rs/aerosocket)
- [Examples](examples/)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT or Apache 2.0
