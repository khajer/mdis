//! Minimal TCP client for the mdis protocol. `std` only — no external
//! crates, same spirit as the Python/Ruby/Go clients.
use std::io::{Read, Write};
use std::net::TcpStream;

mod protocol;
use protocol::{chunk_encode, parse_response, MAX_BUFFER_SIZE};

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
