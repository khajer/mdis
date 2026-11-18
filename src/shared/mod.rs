mod chunked;
mod constants;
mod object;
mod request;

use chrono::Utc;
use std::collections::HashMap;
use std::io::Error;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::error;

use constants::{MAX_BUFFER_SIZE, NEW_LINE_STR, TWO_DELIMITER};
pub use object::ObjectMemory;

pub struct ShareMemory {
    pub data: HashMap<String, ObjectMemory>,
}
impl ShareMemory {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub async fn socket_process(&mut self, socket: &mut TcpStream) {
        let Some((buffer, pos)) = request::read_header(socket).await else {
            error!("Failed to read complete header");
            return;
        };
        let header = String::from_utf8_lossy(&buffer[..pos]).to_string();
        match self.check_header_set_method(header.clone()) {
            Ok(true) => self.call_set_data_process(buffer, pos, header, socket).await,
            Ok(false) => self.call_get_data_process(header, socket).await,
            Err(e) => error!("Failed to check header method; err = {:?}", e),
        }
    }

    pub async fn call_set_data_process(
        &mut self,
        buffer: Vec<u8>,
        header_end: usize,
        header: String,
        socket: &mut TcpStream,
    ) {
        let key_data = header.split_whitespace().nth(1).unwrap().to_string();
        let is_chunked = header.to_lowercase().contains("transfer-encoding: chunked");
        let expire_timeout = request::parse_expire_timeout(&header);

        let raw_data = if is_chunked {
            let remaining = buffer[header_end + 4..].to_vec();
            match chunked::read_chunked_body(socket, remaining).await {
                Some(data) => String::from_utf8_lossy(&data).to_string(),
                None => return,
            }
        } else {
            let mut raw_data = String::from_utf8_lossy(&buffer[header_end + 4..]).to_string();
            if raw_data.ends_with(TWO_DELIMITER) {
                raw_data.truncate(raw_data.len() - 4);
            }
            raw_data
        };

        self.data.insert(
            key_data,
            ObjectMemory {
                duration_sec: expire_timeout,
                raw_data,
                created_at: Utc::now().timestamp(),
            },
        );
        let message_out = format!("OK{NEW_LINE_STR}insert completed{TWO_DELIMITER}");
        let _ = socket.write_all(message_out.as_bytes()).await;
    }

    pub async fn call_get_data_process(&mut self, header: String, socket: &mut TcpStream) {
        let key_data = header.split_whitespace().nth(1).unwrap();
        // get_data() already returns the complete, correctly-framed
        // response (chunked or not) -- send it as-is.
        let string_out = self.get_data(key_data);
        let _ = socket.write_all(string_out.as_bytes()).await;
    }

    pub fn check_header_set_method(&self, header: String) -> Result<bool, Error> {
        request::check_header_set_method(&header)
    }

    pub fn get_data(&mut self, key: &str) -> String {
        let Some(result) = self.data.get(key) else {
            return format!("OK{TWO_DELIMITER}");
        };
        match result.get_key_duration(Utc::now().timestamp()) {
            Some(val) if val.len() > MAX_BUFFER_SIZE => chunked::encode_chunked(&val),
            Some(val) => format!("OK{TWO_DELIMITER}{val}{TWO_DELIMITER}"),
            None => {
                self.data.remove(key);
                format!("Err{NEW_LINE_STR}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(sm: &mut ShareMemory, key: &str, data: String) {
        sm.data.insert(
            key.to_string(),
            ObjectMemory {
                duration_sec: 300,
                raw_data: data,
                created_at: Utc::now().timestamp(),
            },
        );
    }

    #[tokio::test]
    async fn test_recv_n_get_data() {
        let mut sm = ShareMemory::new();
        let test_data = "a".repeat(1000);
        store(&mut sm, "test_large_buffer", test_data.clone());

        assert_eq!(
            sm.get_data("test_large_buffer"),
            format!("OK\r\n\r\n{}\r\n\r\n", test_data)
        );
    }

    #[tokio::test]
    async fn test_recv_n_get_data_chunked() {
        let mut sm = ShareMemory::new();
        let txt_b = "b".repeat(100);
        let txt_a = "a".repeat(MAX_BUFFER_SIZE);
        store(&mut sm, "test_large_buffer", txt_a.clone() + &txt_b);

        assert_eq!(
            sm.get_data("test_large_buffer"),
            format!(
                "OK\r\ntransfer-encoding: chunked\r\n\r\n{:x}\r\n{}\r\n{:x}\r\n{}\r\n0\r\n\r\n",
                MAX_BUFFER_SIZE,
                txt_a,
                txt_b.len(),
                txt_b
            )
        );
    }

    #[tokio::test]
    async fn test_recv_n_get_data_multiple_chunks() {
        let mut sm = ShareMemory::new();
        store(
            &mut sm,
            "test_multiple_chunks",
            "a".repeat(4 * MAX_BUFFER_SIZE + 100),
        );

        let mut expected = "OK\r\ntransfer-encoding: chunked\r\n\r\n".to_string();
        for _ in 0..4 {
            expected.push_str(&format!("{:x}\r\n", MAX_BUFFER_SIZE));
            expected.push_str(&"a".repeat(MAX_BUFFER_SIZE));
            expected.push_str("\r\n");
        }
        expected.push_str(&format!("{:x}\r\n", 100));
        expected.push_str(&"a".repeat(100));
        expected.push_str("\r\n0\r\n\r\n");

        assert_eq!(sm.get_data("test_multiple_chunks"), expected);
    }
}
