//! Minimal TCP client for the mdis protocol. `std` only — no external
//! crates, same spirit as the Python/Ruby/Go clients.
use std::io::{Read, Write};
use std::net::TcpStream;

const MAX_BUFFER_SIZE: usize = 4096;

pub struct MdisClient {
    addr: String,
}

impl MdisClient {
    pub fn connect(host: &str, port: u16) -> Self {
        Self {
            addr: format!("{host}:{port}"),
        }
    }

    /// Store `value` under `key`. `duration` is the optional per-key
    /// expiration in seconds; `0` uses the server's `EXPIRE_TIMEOUT`.
    pub fn set(&self, key: &str, value: &str, duration: i64) -> std::io::Result<String> {
        let mut headers = String::new();
        if duration != 0 {
            headers.push_str(&format!("Duration: {duration}\r\n"));
        }

        let payload = if value.len() > MAX_BUFFER_SIZE {
            headers.push_str("transfer-encoding: chunked\r\n");
            chunk_encode(value)
        } else {
            value.to_string()
        };

        self.send(&format!("set {key}\r\n{headers}\r\n{payload}\r\n\r\n"))
    }

    /// Retrieve the value stored under `key`.
    pub fn get(&self, key: &str) -> std::io::Result<String> {
        self.send(&format!("get {key}\r\n"))
    }

    fn send(&self, message: &str) -> std::io::Result<String> {
        let mut stream = TcpStream::connect(&self.addr)?;
        stream.write_all(message.as_bytes())?;
        stream.shutdown(std::net::Shutdown::Write)?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?; // server closes after replying
        Ok(parse_response(&response))
    }
}

/// Splits `data` into `MAX_BUFFER_SIZE` chunks using the server's hex-size
/// chunked transfer encoding: `{hex_size}\r\n{chunk}\r\n` repeated,
/// terminated by `0\r\n\r\n`.
fn chunk_encode(data: &str) -> String {
    let mut out = String::new();
    for chunk in data.as_bytes().chunks(MAX_BUFFER_SIZE) {
        // Data is plain ASCII in these examples, so byte chunks are valid UTF-8.
        out.push_str(&format!(
            "{:x}\r\n{}\r\n",
            chunk.len(),
            std::str::from_utf8(chunk).unwrap()
        ));
    }
    out.push_str("0\r\n\r\n");
    out
}

const CHUNKED_PREFIX: &str = "OK\r\ntransfer-encoding: chunked\r\n\r\n";

/// Decodes a chunked response body back into the original value. Mirrors
/// `chunk_encode`'s framing: `{hex_size}\r\n{chunk}\r\n` repeated,
/// terminated by `0\r\n\r\n`. `body` is everything after `CHUNKED_PREFIX`.
fn chunk_decode(body: &str) -> String {
    let mut out = String::new();
    let mut offset = 0;
    while let Some(nl) = body[offset..].find("\r\n") {
        let nl = offset + nl;
        let Ok(size) = usize::from_str_radix(&body[offset..nl], 16) else {
            break;
        };
        if size == 0 {
            break;
        }

        let data_start = nl + 2;
        out.push_str(&body[data_start..data_start + size]);
        offset = data_start + size + 2; // skip the chunk's trailing \r\n
    }
    out
}

/// Mirrors the Node.js/Python/Ruby/Go client parsing of the same wire format:
///   SET success:        OK\r\ninsert completed\r\n
///   GET found:           OK\r\n\r\n{data}\r\n\r\n
///   GET not found:        OK\r\n\r\n
///   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
///   error/expired key:    Err\r\n
fn parse_response(data: &str) -> String {
    // A large value comes back chunk-framed rather than as a plain
    // "OK\r\n\r\n{data}\r\n\r\n" body; the chunk data itself may contain
    // \r\n, so it must be decoded from the raw string, not the \r\n-split
    // vec used below for the other response shapes.
    if let Some(body) = data.strip_prefix(CHUNKED_PREFIX) {
        return chunk_decode(body);
    }

    let parts: Vec<&str> = data.split("\r\n").collect();
    let Some(status) = parts.first() else {
        return "NO RESPONSE".to_string();
    };

    match status.to_lowercase().as_str() {
        "ok" => {
            if parts.len() >= 3 && parts[1].is_empty() {
                parts[2].to_string() // GET with a value
            } else if parts.len() >= 2 && !parts[1].is_empty() {
                parts[1].to_string() // SET response message
            } else {
                String::new() // GET, key not found
            }
        }
        "err" => "Error".to_string(),
        _ => "NO RESPONSE".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_set_success() {
        assert_eq!(
            parse_response("OK\r\ninsert completed\r\n"),
            "insert completed"
        );
    }

    #[test]
    fn parses_get_found() {
        assert_eq!(parse_response("OK\r\n\r\nvalue1\r\n\r\n"), "value1");
    }

    #[test]
    fn parses_get_not_found() {
        assert_eq!(parse_response("OK\r\n\r\n"), "");
    }

    #[test]
    fn parses_error() {
        assert_eq!(parse_response("Err\r\n"), "Error");
    }

    #[test]
    fn parses_garbage() {
        assert_eq!(parse_response("garbage"), "NO RESPONSE");
    }

    #[test]
    fn parses_get_found_chunked() {
        let data = "a".repeat(MAX_BUFFER_SIZE + 100);
        let response = format!("{CHUNKED_PREFIX}{}", chunk_encode(&data));
        assert_eq!(parse_response(&response), data);
    }

    #[test]
    fn chunk_round_trip() {
        let data = "a".repeat(MAX_BUFFER_SIZE * 2 + 10);
        let encoded = chunk_encode(&data);

        // Decode using the same rules the server uses, check we get the
        // original bytes back.
        let mut decoded = String::new();
        let mut rest = encoded.as_str();
        loop {
            let nl = rest.find("\r\n").unwrap();
            let size = usize::from_str_radix(&rest[..nl], 16).unwrap();
            if size == 0 {
                break;
            }
            rest = &rest[nl + 2..];
            decoded.push_str(&rest[..size]);
            rest = &rest[size + 2..]; // skip chunk + trailing CRLF
        }
        assert_eq!(decoded, data);
    }
}
