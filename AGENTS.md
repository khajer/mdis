# AGENTS.md

Guidelines for AI coding agents working on the mdis project.

## Project Overview

mdis is a JSON memory cache server written in Rust. It's a TCP socket server that stores data in memory with automatic time-based expiration. Default port: 6411.

## Build & Development Commands

```bash
# Build release binary
cargo build --release

# Run in development (default port 6411)
cargo run

# Run with custom expiration timeout (milliseconds)
EXPIRE_TIMEOUT=10000 cargo run

# Run all tests
cargo test

# Run a single test by name
cargo test test_get_key_duration_success

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt
```

## Code Style Guidelines

### Imports Order
1. Standard library (`std::`)
2. External crates (`tokio::`, `chrono::`, `tracing::`)
3. Local modules (`mod shared;`)

Example:
```rust
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::info;

mod shared;
use shared::ShareMemory;
```

### Naming Conventions
- **Constants**: `UPPER_SNAKE_CASE` with explicit type annotations (e.g., `const HOST:&str = "127.0.0.1:6411";`)
- **Structs/Enums**: `PascalCase` (e.g., `ObjectMemory`, `ShareMemory`)
- **Functions/Methods**: `snake_case` (e.g., `socket_process`, `get_key_duration`)
- **Variables**: `snake_case` (e.g., `shared_memory`, `expire_timeout`)
- **Type parameters**: Single uppercase letters or `PascalCase`

### Types & Documentation
- Use explicit type annotations for constants
- Use `i64` for timestamps and durations
- Use `usize` for buffer sizes and indices
- Document public structs and methods with doc comments

### Error Handling
- Use `Result<T, E>` for fallible operations
- Use `tracing::error!` for logging errors with context
- Prefer `match` over `if let` for complex error handling
- Use `unwrap()` sparingly, only when failure is truly impossible
- Return custom `std::io::Error` for invalid input with descriptive messages

### Async Patterns
- Use `tokio::main` for async entry points
- Use `Arc<Mutex<T>>` for shared state across tasks
- Clone `Arc` before moving into spawned tasks
- Use `tokio::spawn` for concurrent connection handling
- Use `#[tokio::test]` for async tests

### Testing
- Place tests in `#[cfg(test)]` module at end of source files
- Use `#[tokio::test]` for async tests, `#[test]` for sync tests
- Name tests descriptively: `test_<function>_<scenario>_<expected_result>`
- Use `assert_eq!` for exact comparisons
- Use `assert!(true/false)` for boolean conditions
- Use `Utc::now().timestamp()` for time-based tests
- Cover both chunked and non-chunked data paths
- Test expiration logic: `get_key_duration_success`, `get_key_duration_fails_sec`
- Test large data: `recv_n_get_data_chunked`, `recv_n_get_data_multiple_chunks`

### Logging
- Use `tracing::info!` for server lifecycle events (startup, connections)
- Use `tracing::error!` for error conditions with detailed context
- Configure subscriber in `setup_logging()` function

### Protocol Constants

Define delimiter constants as both `&str` and `&[u8]` variants:
```rust
const NEW_LINE_STR: &str = "\r\n";
const NEW_LINE_BYTE: &[u8; 2] = b"\r\n";
const TWO_DELIMITER: &str = "\r\n\r\n";
const TWO_DELIMITER_BYTE: &[u8; 4] = b"\r\n\r\n";
```

- Buffer size: `MAX_BUFFER_SIZE: usize = 4096`
- Default expiration: `EXPIRE_TIMEOUT: i64 = 300` seconds
- Host: `HOST: &str = "127.0.0.1:6411"`

Use string constants for responses, byte constants for socket operations.

## Architecture & Code Organization

### Concurrency Pattern
- Use `Arc<Mutex<ShareMemory>>` for shared state across connections
- Clone `Arc` before moving into spawned tasks: `Arc::clone(&shared_memory)`
- Lock guard is held for the duration of socket processing in each task

### Response Handling
- `OK\r\n` with data: Success response with payload
- `OK\r\ninsert completed\r\n`: SET operation successful
- `OK\r\n\r\n`: GET operation, key not found
- `Err\r\n`: GET operation, key expired (removed from memory)

## Important Gotchas

- **check_header_set_method logic**: The function in src/shared/mod.rs:236 returns `Err` for valid SET headers with < 2 parts. This appears to be a bug - should return `Ok(true)` for SET method regardless of part count.

- **Expiration check timing**: Expired keys are only detected and removed when accessed via GET, not on a background thread.

- **Chunk size calculation**: Chunk sizes are hex-encoded, must be parsed with `usize::from_str_radix(..., 16)`.

- **Data truncation**: Non-chunked data with trailing `\r\n\r\n` has this delimiter stripped (lines 194-196 in src/shared/mod.rs).

- **Lock granularity**: Each connection holds the lock for the entire socket processing duration, which may cause contention under high load.

- **No connection pooling**: Each operation creates a new socket connection in client examples.

## Project Structure

```
src/
├── main.rs           # Entry point, TCP listener, connection handling
└── shared/
    └── mod.rs        # ShareMemory, ObjectMemory, protocol implementation

examples/
├── nodejs/
│   └── simple/       # Node.js client library and examples
└── python/
    └── simple/       # Python client library and examples
```

## Dependencies

- `tokio` - Async runtime (full features)
- `chrono` - Timestamp management with serde feature
- `tracing` / `tracing-subscriber` - Structured logging

## Environment Variables

- `EXPIRE_TIMEOUT` - Default expiration time in seconds (default: 300). Can be set per-key via `Duration:` header in SET requests.
- Values are parsed as `i64` - will fall back to default if parsing fails.

```bash
# Set global default to 10 seconds
EXPIRE_TIMEOUT=10 cargo run

# Set per-key expiration via protocol header
# "Duration: 60\r\n" in SET request for 60-second TTL
```

## Docker Commands

```bash
docker build -t mdis .
docker run -p 6411:6411 mdis
docker run -p 6411:6411 -e EXPIRE_TIMEOUT=10000 mdis
```

## Protocol Specification

mdis uses a custom TCP protocol with HTTP-style headers and chunked transfer encoding.

### Message Format
- Headers end with double CRLF: `\r\n\r\n`
- First line format: `METHOD key`
- Optional headers follow (one per line)

### Methods

**SET** - Store data with optional duration:
```
SET {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{data}\r\n\r\n
```

- `Duration` header overrides `EXPIRE_TIMEOUT` for this key only
- `transfer-encoding: chunked` required for payloads > 4096 bytes
- Chunked data format: `{hex_size}\r\n{chunk_data}\r\n` (repeat) + `0\r\n\r\n`

**GET** - Retrieve data by key:
```
GET {key}\r\n
```

### Response Formats

- **SET success**: `OK\r\ninsert completed\r\n`
- **GET success (data found)**: `OK\r\n\r\n{data}\r\n\r\n` (or chunked for large data)
- **GET not found**: `OK\r\n\r\n`
- **GET expired**: `Err\r\n` (key automatically removed)

### Important Behaviors

- Trailing `\r\n\r\n` is automatically stripped from non-chunked SET data
- Expired keys are removed on GET attempts
- Chunked encoding is mandatory for payloads > `MAX_BUFFER_SIZE` (4096 bytes)
- Chunk sizes are in hexadecimal

## Client Library Testing

```bash
# Node.js examples
node examples/nodejs/simple/example_1.js  # SET operation
node examples/nodejs/simple/example_2.js  # GET operation

# Python examples
python examples/python/simple/example_1.py  # SET operation
python examples/python/simple/example_2.py  # GET operation
```
