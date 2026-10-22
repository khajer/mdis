use std::env;
use std::io::Error;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::error;

use super::constants::{
    EXPIRE_TIMEOUT, MAX_BUFFER_SIZE, NEW_LINE_BYTE, NEW_LINE_STR, TWO_DELIMITER_BYTE,
};

/// Read from the socket until the header is complete.
/// Returns the buffered bytes and the header end position.
pub async fn read_header(socket: &mut TcpStream) -> Option<(Vec<u8>, usize)> {
    let mut buf = [0; MAX_BUFFER_SIZE];
    let mut buffer = Vec::new();
    loop {
        match socket.read(&mut buf).await {
            Ok(0) => return None,
            Ok(n) => {
                buffer.extend_from_slice(&buf[..n]);

                if let Some(pos) = buffer.windows(4).position(|w| w == TWO_DELIMITER_BYTE) {
                    return Some((buffer, pos));
                }

                // GET requests carry no headers or body -- per the
                // protocol spec they end after a single "GET {key}\r\n"
                // line, so don't block waiting for a second CRLF that
                // a well-behaved GET client will never send.
                if let Some(pos) = buffer.windows(2).position(|w| w == NEW_LINE_BYTE) {
                    let first_line = String::from_utf8_lossy(&buffer[..pos]);
                    let is_get = first_line
                        .split_whitespace()
                        .next()
                        .is_some_and(|m| m.eq_ignore_ascii_case("get"));
                    if is_get {
                        return Some((buffer, pos));
                    }
                }
            }
            Err(e) => {
                error!("Failed to read from socket; err = {:?}", e);
                return None;
            }
        }
    }
}

/// Resolve the expiration: env default, overridden by a `Duration:` header.
pub fn parse_expire_timeout(header: &str) -> i64 {
    let mut expire_timeout = env::var("EXPIRE_TIMEOUT")
        .unwrap_or(EXPIRE_TIMEOUT.to_string())
        .parse::<i64>()
        .unwrap_or(EXPIRE_TIMEOUT);

    for line in header.split(NEW_LINE_STR).skip(1) {
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() == 2 && parts[0].to_lowercase() == "duration:" {
            if let Ok(duration) = parts[1].parse::<i64>() {
                expire_timeout = duration;
            }
        }
    }
    expire_timeout
}

/// Returns `Ok(true)` for SET, `Ok(false)` for GET.
pub fn check_header_set_method(header: &str) -> Result<bool, Error> {
    let first_line = header.split(NEW_LINE_STR).next().unwrap_or("");
    match first_line.split(' ').next().map(|m| m.to_lowercase()) {
        Some(m) if m == "set" => Ok(true),
        Some(m) if m == "get" => Ok(false),
        _ => Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid header format",
        )),
    }
}
