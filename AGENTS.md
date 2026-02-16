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
- Name tests descriptively: `test_<function>_<scenario>_<expected_result>`
- Use `assert_eq!` for exact comparisons
- Use `assert!(true/false)` for boolean conditions
- Use `Utc::now().timestamp()` for time-based tests
- Cover both chunked and non-chunked data paths

### Logging
- Use `tracing::info!` for server lifecycle events (startup, connections)
- Use `tracing::error!` for error conditions with detailed context
- Configure subscriber in `setup_logging()` function

### Protocol Constants
- Define delimiter constants as both `&str` and `&[u8]` variants:
```rust
const NEW_LINE_STR: &str = "\r\n";
const NEW_LINE_BYTE: &[u8; 2] = b"\r\n";
```
- Buffer size: `MAX_BUFFER_SIZE: usize = 4096`
- Default expiration: `EXPIRE_TIMEOUT: i64 = 300` seconds

## Project Structure

```
src/
├── main.rs           # Entry point, TCP listener, connection handling
└── shared/
    └── mod.rs        # ShareMemory, ObjectMemory, protocol implementation
```

## Dependencies

- `tokio` - Async runtime (full features)
- `chrono` - Timestamp management
- `tracing` / `tracing-subscriber` - Structured logging

## Docker Commands

```bash
docker build -t mdis .
docker run -p 6411:6411 mdis
docker run -p 6411:6411 -e EXPIRE_TIMEOUT=10000 mdis
```
