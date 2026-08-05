# MDIS Client for Rust

A minimal Rust client for the mdis server. `std` only (`std::net::TcpStream`)
— no external crates, same spirit as the Python/Ruby/Go clients.

## Project Structure

```
examples/rust/simple/
├── src/
│   └── lib.rs          # MdisClient implementation + unit tests
├── examples/
│   ├── example_1.rs    # SET operation
│   ├── example_2.rs    # GET operation
│   ├── example_3.rs    # SET with per-key expiration
│   └── example_4.rs    # SET with a payload > 4096 bytes (chunked)
├── Cargo.toml
└── README.md
```

`examples/` here is Cargo's own convention for runnable usage samples of a
library crate (`cargo run --example <name>`).

## Quick Start

```rust
use mdis_client_example::MdisClient;

let client = MdisClient::connect("127.0.0.1", 6411);

let resp = client.set("token", "123456", 0)?; // duration=0 -> server default TTL
let token = client.get("token")?;
```

## Running the examples

```bash
cd examples/rust/simple
cargo run --example example_1   # SET
cargo run --example example_2   # GET
cargo run --example example_3   # SET with 2s expiration, GET after it lapses
cargo run --example example_4   # SET a 5000-byte payload (chunked transfer)
```

## Testing

```bash
cargo test
```

Covers response parsing and the chunked-encoding round trip.

## API Reference

- `MdisClient::connect(host: &str, port: u16) -> MdisClient`
- `MdisClient::set(&self, key: &str, value: &str, duration: i64) -> std::io::Result<String>` — `duration` in seconds, `0` for the server default
- `MdisClient::get(&self, key: &str) -> std::io::Result<String>`

## Protocol

Same wire protocol as the other client examples — see the top-level
`AGENTS.md` for the full spec. In short:

```
SET {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{data}\r\n\r\n
GET {key}\r\n
```

Payloads over 4096 bytes are sent using hex-size chunked encoding
(`{hex_size}\r\n{chunk}\r\n` repeated, terminated by `0\r\n\r\n`).
