<div align="center">
  <img src="assets/banner.svg" alt="AeroSocket Banner" width="800" height="200">
</div>

# AeroSocket

[![Crates.io](https://img.shields.io/crates/v/aerosocket.svg)](https://crates.io/crates/aerosocket)
[![Documentation](https://docs.rs/aerosocket/badge.svg)](https://docs.rs/aerosocket)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/discord/6nS2KqxQtj?label=discord)](https://discord.gg/6nS2KqxQtj)

A fast, zero-copy WebSocket library for Rust.

We built AeroSocket because we needed something that could handle millions of concurrent connections without falling over. Most existing libraries either sacrificed performance for ergonomics or were a pain to actually use. We wanted both.

---

## What makes it different?

**It's fast.** Like, really fast. Zero-copy message handling means we're not wasting cycles shuffling bytes around. On our benchmarks we're hitting 2.5M messages per second on modest hardware.

**It doesn't get in your way.** The API is designed to feel natural. Builder patterns where they make sense, sensible defaults so you're not drowning in configuration.

**It's built for production.** Rate limiting, TLS, graceful shutdown, proper error handling—the stuff you actually need when you're running this at scale.

---

## Getting started

Here's a basic echo server:

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
aerosocket = { version = "0.2", features = ["full"] }
tokio = { version = "1.0", features = ["full"] }
```

---

## Performance

We ran these on an AWS c6i.large with Rust 1.75 and 10k concurrent connections:

| Metric | AeroSocket | tokio-tungstenite | fastwebsockets |
|--------|------------|-------------------|----------------|
| Throughput (small msgs) | 2.5M msg/s | 1.2M msg/s | 1.8M msg/s |
| Latency P99 | < 50μs | 120μs | 80μs |
| Memory usage | 40% less | baseline | 25% less |
| Zero-copy | Yes | No | Yes |

Your mileage may vary, but these numbers have held up pretty well across different workloads.

---

## How it works

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

The zero-copy engine is the secret sauce. Instead of copying message payloads at every layer, we pass around references and only allocate when absolutely necessary. This matters a lot when you're pushing millions of messages.

---

## Who's using this?

We've seen it work well for:

- **Chat apps** — Slack-style messaging, collaborative editors, that kind of thing
- **Trading platforms** — Real-time market data where latency matters
- **Game servers** — Multiplayer backends, leaderboards, matchmaking
- **IoT** — Sensor networks, monitoring dashboards, alerting systems

Basically anywhere you need to move a lot of data to a lot of clients quickly.

---

## What's included

**Working right now:**
- Full RFC6455 WebSocket implementation
- TCP and TLS transports
- Rate limiting (per-IP request and connection limits)
- Structured logging via tracing
- Graceful shutdown
- Backpressure handling

**Still working on:**
- Authentication framework
- Prometheus metrics
- Health check endpoints
- Per-message compression
- CORS handling

**On the roadmap:**
- QUIC transport
- Load balancing
- Kubernetes operator

---

## Feature flags

Use what you need:

```toml
[dependencies]
aerosocket = { version = "0.2", features = ["full"] }
```

### Minimal Installation

```toml
[dependencies]
aerosocket = { version = "0.2", default-features = false, features = ["transport-tcp", "tokio-runtime"] }
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

**Zero-copy sends:**

```rust
let data = Bytes::from("large payload");
conn.send_binary_bytes(data).await?; // No copy here
```

**Connection pooling (planned):**

Client-side connection pooling is on the roadmap for a future release.

### Server-side WASM handlers (preview)

If you enable the `wasm-handlers` feature on `aerosocket-server`, you can host
WASM-based connection handlers alongside native Rust handlers:

```toml
[dependencies]
aerosocket-server = { version = "0.2", features = ["full"] }
```

On the server side, `ServerBuilder` exposes a helper to load a WASM handler from
disk (requires `wasm-handlers`):

```rust
use aerosocket_server::prelude::*;

let server = Server::builder()
    .bind("0.0.0.0:8080")?
    .build_with_wasm_handler_from_file(
        "aerosocket-wasm-handler/target/wasm32-wasi/release/aerosocket-wasm-handler.wasm",
    )?;
```

WASM modules use a very small ABI:

- Export linear memory (the default `memory` export in Rust is fine).
- Export a function `on_message(ptr: i32, len: i32) -> i32`.
  - Host writes UTF-8 text into linear memory at `ptr..ptr+len`.
  - The WASM function may transform the bytes in-place and returns the number of
    bytes written.
  - Host reads back that many bytes and sends them as the outgoing text frame.

This repo includes two small example crates you can copy or adapt:

- `aerosocket-wasm-handler` — simple text transformer that prefixes messages
  with `"WASM: "`.
- `aerosocket-wasm-json-handler` — JSON-aware handler that parses the incoming
  text as JSON, wraps it in a small metadata envelope, and serializes it back to
  JSON.

---

## Security

We take this seriously. Here's what's in place:

- TLS 1.2/1.3 with configurable root CAs and SNI
- Rate limiting and connection quotas
- Input validation for malformed frames
- Memory safety (it's Rust, so this comes for free)
- Backpressure to prevent resource exhaustion

Production config looks something like:

```rust
use aerosocket::prelude::*;

let server = aerosocket::Server::builder()
    .bind("0.0.0.0:8443")?
    .max_connections(10_000)
    .compression(true)
    .backpressure(BackpressureStrategy::Buffer)
    .tls("server.crt", "server.key")
    .transport_tls()
    .idle_timeout(Duration::from_secs(300))
    .build()?;
```

---

## Documentation

- [Getting Started](docs/getting-started.md)
- [API Reference](https://docs.rs/aerosocket)
- [Examples](examples/)
- [Performance Tuning](docs/performance.md)
- [Security Guide](docs/security.md)

---

## Contributing

We'd love help. Clone the repo and poke around:

```bash
git clone https://github.com/M1tsumi/AeroSocket
```

Check out [CONTRIBUTING.md](CONTRIBUTING.md) for the details.

**Get in touch:**
- [Discord](https://discord.gg/6nS2KqxQtj)
- [GitHub Issues](https://github.com/M1tsumi/AeroSocket/issues)
- [Discussions](https://github.com/M1tsumi/AeroSocket/discussions)

---

## 🗺️ Roadmap

### ✅ **v0.1 (done)**
- Core WebSocket protocol
- TCP and TLS transports
- Rate limiting
- Logging
- Connection management
- Zero-copy engine

**v0.2 (released)**
- Authentication
- Metrics
- Health checks
- Compression
- CORS

**v0.3 (planned)**
- HTTP/2 transport
- Better connection pooling
- WASM support

**v1.0 (eventually)**
- QUIC
- Load balancing
- Kubernetes integration

---

## Ecosystem

| Integration | Status |
|-------------|--------|
| Tokio | Works |
| Serde | Works |
| Tracing | Built-in |
| Tower | In progress |
| Axum | In progress |
| Actix | Planned |

---

## License

MIT or Apache 2.0, your choice.

---

## Thanks

This wouldn't exist without the Tokio team's work on the async runtime, or the folks who wrote the WebSocket RFC. And thanks to everyone who's tried early versions and told us what was broken.

---

<div align="center">

If you find this useful, a star on GitHub helps others find it too.

[![GitHub stars](https://img.shields.io/github/stars/M1tsumi/AeroSocket?style=social)](https://github.com/M1tsumi/AeroSocket)

</div>
