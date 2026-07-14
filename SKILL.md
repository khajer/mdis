# SKILL.md

This document outlines the technical skills and knowledge required to effectively work on the mdis project - a JSON memory cache server written in Rust.

## Core Technical Skills

### Rust Programming Language

**Essential Knowledge:**
- Strong understanding of Rust's ownership, borrowing, and lifetime system
- Proficiency with Rust's type system, including structs, enums, and traits
- Error handling using `Result<T, E>` and `Option<T>`
- Pattern matching with `match`, `if let`, and `while let`
- Understanding of safe vs unsafe Rust (mdis uses safe Rust exclusively)

**Key Concepts Applied:**
```rust
// Example: Thread-safe shared state
use std::sync::Arc;
use tokio::sync::Mutex;

// Arc enables multiple ownership across threads
// Mutex ensures exclusive access to the data
let shared_memory = Arc::new(Mutex::new(ShareMemory::new()));
```

### Async Programming with Tokio

**Required Knowledge:**
- Understanding of Rust's async/await syntax
- Tokio runtime and task spawning with `tokio::spawn`
- Working with asynchronous I/O and TCP networking
- Handling cancellation and timeouts in async contexts

**Key Patterns Used:**
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&HOST).await?;
    
    loop {
        let (socket, _) = listener.accept().await?;
        let shared = Arc::clone(&shared_memory);
        
        tokio::spawn(async move {
            socket_process(socket, shared).await;
        });
    }
}
```

### Network Programming

**TCP Socket Programming:**
- Creating and managing TCP connections
- Reading and writing to network streams
- Handling partial reads and buffer management
- Implementing custom network protocols

**Protocol Design:**
- Understanding of HTTP-style protocol design
- Header parsing with CRLF delimiters
- Chunked transfer encoding for large payloads
- State management for multi-step protocol interactions

### Concurrent Programming

**Thread-Safe Data Structures:**
- `Arc<T>` for shared ownership across threads
- `Mutex<T>` for exclusive access to shared data
- Cloning Arc references before moving into spawned tasks
- Understanding of deadlock prevention strategies

**Implementation:**
```rust
// Pattern for cloning Arc before moving into task
let shared_clone = Arc::clone(&shared);
tokio::spawn(async move {
    // shared_clone is now owned by this task
    // Can safely access shared data through the Mutex
    let mut data = shared_clone.lock().await;
    // Modify data...
});
```

## Domain Knowledge

### Caching Systems

**Core Concepts:**
- In-memory caching strategies
- Time-based expiration mechanisms
- Key-value store design patterns
- Cache invalidation strategies

**Implementation in mdis:**
```rust
// Expiration check logic
pub fn get_key_duration(&self) -> Result<i64, std::io::Error> {
    let now = Utc::now().timestamp();
    let expire_time = self.created_at + self.duration_sec;
    
    if now > expire_time {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Key has expired"
        ))
    } else {
        Ok(expire_time - now)
    }
}
```

### Data Serialization

**JSON Handling:**
- Storing and retrieving JSON data as strings
- Understanding of data format validation
- Handling of escape sequences and special characters

### Time Management

**Timestamps and Durations:**
- Using `chrono` crate for UTC timestamps
- Calculating time differences for expiration
- Converting between time units (seconds, milliseconds)
- Environment variable configuration for timeouts

## Testing Skills

### Unit Testing in Rust

**Testing Patterns:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_key_duration_success() {
        let obj = ObjectMemory {
            raw_data: String::from("test"),
            duration_sec: 300,
            created_at: Utc::now().timestamp(),
        };
        
        assert!(obj.get_key_duration().is_ok());
    }
}
```

**Test Coverage Requirements:**
- Happy path testing (successful operations)
- Error case testing (expired keys, invalid input)
- Edge case testing (boundary conditions, empty data)
- Chunked vs non-chunked transfer encoding paths
- Concurrent access scenarios

### Integration Testing

**Skills Needed:**
- Testing TCP socket connections
- Protocol message construction
- Multi-client concurrent access testing
- Performance testing for cache operations

## Architecture Understanding

### Protocol Design

**Message Structure:**
```
METHOD key\r\n
Header: value\r\n
\r\n
[payload body]
```

**Chunked Encoding:**
```
4\r\n
data\r\n
4\r\n
more\r\n
0\r\n
\r\n
```

### Code Organization

**Module Structure:**
- `main.rs`: Entry point, server setup, connection handling
- `shared/mod.rs`: Core business logic, protocol implementation
- Clear separation of concerns between networking and data management

### Performance Considerations

**Optimization Knowledge:**
- Buffer size management (MAX_BUFFER_SIZE = 4096)
- Efficient string handling and copying
- Lock contention minimization in concurrent access
- Memory management for long-running server processes

## Best Practices

### Code Style

**Formatting Standards:**
- Use `cargo fmt` for consistent code formatting
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Import organization: std, external crates, local modules
- Clear and descriptive variable names

### Documentation

**Requirements:**
- Public structs and methods must have doc comments
- Complex algorithms need inline explanations
- Protocol specifications should be well-documented
- Include usage examples in documentation

### Error Handling

**Patterns:**
```rust
// Descriptive error messages
return Err(std::io::Error::new(
    std::io::ErrorKind::InvalidInput,
    "Invalid method in request header"
));

// Logging with context
tracing::error!("Failed to read from socket: {:?}", error);
```

### Logging

**Tracing Integration:**
- Structured logging with tracing crate
- Different log levels (info, error, debug)
- Contextual information in log messages
- Performance monitoring through logs

## Additional Skills

### Docker and Containerization

**Knowledge Required:**
- Writing Dockerfiles for Rust applications
- Multi-stage builds for optimized image size
- Environment variable configuration
- Port mapping and networking basics

### Git Workflow

**Essential Git Skills:**
- Branching strategies for feature development
- Commit message conventions
- Pull request processes
- Handling merge conflicts

### Debugging

**Debugging Techniques:**
- Using `cargo test -- --nocapture` for test output
- Adding debug logging with `tracing::debug!`
- Using `cargo check` for early error detection
- Understanding async stack traces

## Learning Path

### For New Contributors

1. **Week 1: Rust Fundamentals**
   - Complete Rust ownership and borrowing exercises
   - Practice with basic async/await patterns
   - Review existing codebase structure

2. **Week 2: Network Programming**
   - Study TCP socket programming in Rust
   - Implement a simple echo server
   - Practice custom protocol design

3. **Week 3: Testing and Debugging**
   - Write comprehensive unit tests
   - Practice debugging async code
   - Learn performance profiling techniques

4. **Week 4: Advanced Topics**
   - Study concurrent programming patterns
   - Implement caching strategies
   - Practice with Docker containerization

### Recommended Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **Tokio Documentation**: https://tokio.rs/
- **Rust Async Book**: https://rust-lang.github.io/async-book/
- **Chrono Crate**: https://docs.rs/chrono/

## Project-Specific Knowledge

### Environment Variables

- `EXPIRE_TIMEOUT`: Default key expiration time in seconds (default: 300)
- `HOST`: Server bind address (default: "127.0.0.1:6411")

### Constants

- `MAX_BUFFER_SIZE`: 4096 bytes (threshold for chunked encoding)
- `NEW_LINE_STR`: "\r\n" (protocol delimiter)
- `NEW_LINE_BYTE`: b"\r\n" (byte representation)

### Client Protocol Implementation

Client libraries implement:
- TCP connection establishment
- Protocol message construction
- Chunked data handling
- Response parsing

## Conclusion

Working on mdis requires a solid foundation in Rust programming, async programming with Tokio, and network programming concepts. The codebase demonstrates best practices in concurrent programming, protocol design, and memory caching systems. Mastery of these skills will enable effective contribution to the project and understanding of its architecture.

For specific implementation questions, refer to the inline documentation in the source code and the comprehensive test suite which serves as both verification and example code.