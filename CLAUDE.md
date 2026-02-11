# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mdis is a JSON memory cache server written in Rust. It's a TCP socket server that stores data in memory with automatic time-based expiration. Default port is 6411.

## Development Commands

### Building and Running
- Build release binary: `cargo build --release`
- Run in development: `cargo run`
- Run with custom expiration timeout: `EXPIRE_TIMEOUT=10000 cargo run`

### Testing
- Run all tests: `cargo test`
- Run specific test: `cargo test test_name`
- Run with output: `cargo test -- --nocapture`

### Docker
- Build image: `docker build -t mdis .`
- Run container: `docker run -p 6411:6411 mdis`
- Run with custom timeout: `docker run -p 6411:6411 -e EXPIRE_TIMEOUT=10000 mdis`

### Testing with Client Examples
- Node.js examples: `node examples/nodejs/simple/example_1.js`
- Python examples: `python examples/python/simple/example_1.py`

## Architecture

### Core Components

**main.rs** (src/main.rs:1)
- Entry point that sets up TCP listener on 127.0.0.1:6411
- Uses tokio async runtime for concurrent connection handling
- Each connection spawns a new task with cloned Arc<Mutex<ShareMemory>>
- Logging configured via tracing/tracing-subscriber

**ShareMemory** (src/shared/mod.rs:36)
- Core data structure: HashMap<String, ObjectMemory>
- Thread-safe via Arc<Mutex> wrapper in main
- Handles socket protocol parsing and response generation

**ObjectMemory** (src/shared/mod.rs:11)
- Stores: raw_data (String), duration_sec (i64), created_at (timestamp)
- Automatic expiration via get_key_duration() method

### Protocol Design

mdis uses a custom TCP protocol with these characteristics:

**Message Format**
- Headers end with double CRLF: `\r\n\r\n`
- First line format: `METHOD key`
- Optional headers follow (e.g., `Duration: 300`)

**Methods**
- SET: Stores data with key and optional duration
- GET: Retrieves data by key

**Transfer Encoding**
- Regular: For payloads ≤ 4096 bytes
- Chunked: For payloads > 4096 bytes (MAX_BUFFER_SIZE)
- Chunked format follows HTTP-style encoding with hex chunk sizes

**Expiration**
- Default: 300 seconds (EXPIRE_TIMEOUT constant)
- Configurable via EXPIRE_TIMEOUT environment variable
- Can be set per-key via Duration header in SET requests
- Expired keys return error on GET and are removed from memory

### Key Implementation Details

**socket_process** (src/shared/mod.rs:46)
- Reads from socket until finding double CRLF delimiter
- Parses headers to determine SET vs GET method
- Delegates to call_set_data_process or call_get_data_process

**call_set_data_process** (src/shared/mod.rs:93)
- Handles both chunked and non-chunked data
- Parses Duration header to override default timeout
- Stores ObjectMemory in HashMap with current timestamp

**call_get_data_process** (src/shared/mod.rs:209)
- Retrieves data and checks expiration via get_key_duration
- Returns chunked response for large data
- Automatically removes expired entries

**get_data** (src/shared/mod.rs:259)
- Core retrieval logic with expiration checking
- Generates chunked responses for data > MAX_BUFFER_SIZE
- Returns different formats: "OK" (found), "Err" (expired), "OK\r\n\r\n" (not found)

## Testing Strategy

Tests are located in src/shared/mod.rs starting at line 308. Focus areas:

- **Expiration logic**: test_get_key_duration_* tests verify timeout behavior
- **Chunked encoding**: test_recv_n_get_data_chunked tests verify chunking for payloads > 4096 bytes
- **Multiple chunks**: test_recv_n_get_data_multiple_chunks tests large data handling

When adding features, ensure tests cover both chunked and non-chunked code paths.

## Dependencies

- **tokio**: Async runtime with full features
- **chrono**: Timestamp management for expiration
- **tracing/tracing-subscriber**: Structured logging

## Client Libraries

Client implementations are in examples/ directory:
- Node.js client: examples/nodejs/simple/
- Python client: examples/python/simple/

Both implement the mdis protocol for SET/GET operations with chunked transfer support.
