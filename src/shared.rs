use chrono::Utc;
use std::collections::HashMap;
use std::env;
use std::io::Error;

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::TcpStream;

pub struct ObjectMemory {
    pub raw_data: String,
    pub duration_sec: i64,
    pub created_at: i64,
}
const MAX_BUFFER_SIZE: usize = 4096;
const EXPIRE_TIMEOUT: i64 = 300;

impl ObjectMemory {
    pub fn get_key_duration(&self, curr_time: i64) -> Option<String> {
        let duration = curr_time - self.created_at;
        if duration > self.duration_sec {
            None
        } else {
            Some(self.raw_data.clone())
        }
    }
}

pub struct ShareMemory {
    pub data: HashMap<String, ObjectMemory>,
}
impl ShareMemory {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub async fn recv_data(&mut self, socket: &mut TcpStream) -> Result<String, Error> {
        let mut buf = [0; MAX_BUFFER_SIZE];

        match socket.read(&mut buf).await {
            Ok(0) => {
                return Ok("".to_string());
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buf[..n]).to_string();
                let response = self.receive_message(message);
                Ok(response)
            }
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                Err(e)
            }
        }
    }

    pub async fn recv_data_raw<R>(&mut self, reader: &mut R) -> Result<ObjectMemory, Error>
    where
        R: AsyncRead + Unpin,
    {
        let mut buf = [0; MAX_BUFFER_SIZE];

        let mut header_end = None;
        let mut buffer = Vec::new();

        while header_end.is_none() {
            match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&buf[..n]);
                    if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                        header_end = Some(pos);
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        let header_end = match header_end {
            Some(pos) => pos,
            None => {
                return Ok(ObjectMemory {
                    duration_sec: 300,
                    raw_data: String::from_utf8_lossy(&buffer).to_string(),
                    created_at: Utc::now().timestamp(),
                });
            }
        };

        let header_bytes = &buffer[..header_end];
        let header = String::from_utf8_lossy(header_bytes);

        let is_chunked = header.to_lowercase().contains("transfer-encoding: chunked");
        if is_chunked {
            let mut remaining_data = Vec::new();
            remaining_data.extend_from_slice(&buffer[header_end + 4..]);
            let mut complete_data = Vec::new();

            loop {
                let chunk_size_end = match remaining_data.windows(2).position(|w| w == b"\r\n") {
                    Some(pos) => pos,
                    None => match reader.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            remaining_data.extend_from_slice(&buf[..n]);
                            continue;
                        }
                        Err(e) => return Err(e),
                    },
                };

                let chunk_size_str = String::from_utf8_lossy(&remaining_data[..chunk_size_end]);
                let chunk_size = match usize::from_str_radix(chunk_size_str.trim(), 16) {
                    Ok(size) => size,
                    Err(_) => {
                        break;
                    }
                };

                if chunk_size == 0 {
                    break;
                }

                let chunk_data_start = chunk_size_end + 2;
                let chunk_data_end = chunk_data_start + chunk_size;
                if remaining_data.len() < chunk_data_end {
                    let needed = chunk_data_end - remaining_data.len();
                    let mut extra_buf = vec![0; needed];
                    match reader.read_exact(&mut extra_buf).await {
                        Ok(_) => {
                            remaining_data.extend_from_slice(&extra_buf);
                        }
                        Err(e) => return Err(e),
                    }
                }

                let trailing_crlf_start = chunk_data_end;
                let trailing_crlf_end = trailing_crlf_start + 2;

                let chunk_data = &remaining_data[chunk_data_start..chunk_data_end];
                complete_data.extend_from_slice(chunk_data);

                if remaining_data.len() < trailing_crlf_end {
                    let needed = trailing_crlf_end - remaining_data.len();
                    let mut extra_buf = vec![0; needed];
                    match reader.read_exact(&mut extra_buf).await {
                        Ok(_) => {
                            remaining_data.extend_from_slice(&extra_buf);
                        }
                        Err(e) => return Err(e),
                    }
                }

                if remaining_data.len() > trailing_crlf_end {
                    remaining_data = remaining_data[trailing_crlf_end..].to_vec();
                } else {
                    remaining_data.clear();
                }
            }

            Ok(ObjectMemory {
                duration_sec: 300,
                raw_data: String::from_utf8_lossy(&complete_data).to_string(),
                created_at: Utc::now().timestamp(),
            })
        } else {
            let mut raw_data = String::from_utf8_lossy(&buffer[header_end + 4..]).to_string();

            if raw_data.ends_with("\r\n\r\n") {
                raw_data.truncate(raw_data.len() - 4);
            }

            Ok(ObjectMemory {
                duration_sec: 300,
                raw_data,
                created_at: Utc::now().timestamp(),
            })
        }
    }

    pub fn receive_message(&mut self, message: String) -> String {
        if message.contains("\r\n\r\n") {
            let message_parts: Vec<&str> = message.split("\r\n\r\n").collect();
            if message_parts.len() < 2 {
                return "Err\r\n".to_string();
            }

            let header_part = message_parts[0];
            let data_part = message_parts[1];

            let header_lines: Vec<&str> = header_part.split("\r\n").collect();
            if header_lines.is_empty() {
                return "Err\r\n\r\n".to_string();
            }

            let first_line_parts: Vec<&str> = header_lines[0].split(' ').collect();
            if first_line_parts.len() < 2 {
                return "Err\r\n\r\n".to_string();
            }

            let method_name = first_line_parts[0].to_string().to_lowercase();
            let key_data = first_line_parts[1].to_string();

            if method_name == "set" {
                let mut expire_timeout = env::var("EXPIRE_TIMEOUT")
                    .unwrap_or(EXPIRE_TIMEOUT.to_string())
                    .parse::<i64>()
                    .unwrap_or(EXPIRE_TIMEOUT);

                for line in header_lines.iter().skip(1) {
                    let parts: Vec<&str> = line.split(' ').collect();
                    if parts.len() == 2 && parts[0].to_lowercase() == "duration:" {
                        if let Ok(duration) = parts[1].parse::<i64>() {
                            expire_timeout = duration;
                        }
                    }
                }

                let trimmed_data = if data_part.ends_with("\r\n") {
                    &data_part[..data_part.len() - 2]
                } else {
                    data_part
                };

                self.data.insert(
                    key_data,
                    ObjectMemory {
                        raw_data: trimmed_data.to_string(),
                        duration_sec: expire_timeout,
                        created_at: Utc::now().timestamp(),
                    },
                );

                return "OK\r\ninsert completed\r\n\r\n".to_string();
            } else if method_name == "get" {
                match self.data.get(&key_data) {
                    Some(result) => {
                        if let Some(v) = result.get_key_duration(Utc::now().timestamp()) {
                            return "OK\r\n\r\n".to_string() + &v + "\r\n\r\n";
                        } else {
                            self.data.remove(&key_data);
                            return "Err\r\n".to_string();
                        }
                    }
                    None => {
                        return "OK\r\n\r\n".to_string();
                    }
                }
            } else {
                return "Err\r\n\r\n".to_string();
            }
        } else {
            let parts: Vec<&str> = message.split("\r\n").collect();
            let header = parts[0];
            let header_message: Vec<&str> = header.split(' ').collect();

            if header_message.len() >= 2 {
                let method_name = header_message[0].to_string().to_lowercase();
                if method_name == "set" {
                    let key_data = header_message[1].to_string();

                    let mut expire_timeout = env::var("EXPIRE_TIMEOUT")
                        .unwrap_or(EXPIRE_TIMEOUT.to_string())
                        .parse::<i64>()
                        .unwrap_or(EXPIRE_TIMEOUT);
                    let mut value_line = 2;

                    if parts.len() > 2 && !parts[1].is_empty() {
                        let duration_parts: Vec<&str> = parts[1].split(' ').collect();
                        if duration_parts.len() == 2
                            && duration_parts[0].to_lowercase() == "duration:"
                        {
                            //set duration
                            let duration_str = duration_parts[1];
                            if let Ok(duration) = duration_str.parse::<i64>() {
                                expire_timeout = duration;
                            }
                            value_line = 3;
                        } else {
                            return "Err\r\n\r\n".to_string();
                        }
                    }

                    if parts.len() <= value_line || !parts[value_line - 1].is_empty() {
                        return "Err\r\n\r\n".to_string();
                    }

                    let value = parts.get(value_line).unwrap_or(&"").to_string();

                    self.data.insert(
                        key_data,
                        ObjectMemory {
                            raw_data: value,
                            duration_sec: expire_timeout,
                            created_at: Utc::now().timestamp(),
                        },
                    );

                    return "OK\r\ninsert completed\r\n\r\n".to_string();
                } else if method_name == "get" {
                    let key_data = header_message[1].to_string();
                    match self.data.get(&key_data) {
                        Some(result) => {
                            if let Some(v) = result.get_key_duration(Utc::now().timestamp()) {
                                return "OK\r\n\r\n".to_string() + &v + "\r\n\r\n";
                            } else {
                                self.data.remove(&key_data);
                                return "Err\r\n".to_string();
                            }
                        }
                        None => {
                            return "OK\r\n\r\n".to_string();
                        }
                    }
                } else {
                    return "Err\r\n\r\n".to_string();
                }
            }

            "Err\r\n\r\n".to_string()
        }
    }

    pub fn get_data(&mut self, key: &str) -> String {
        match self.data.get(key) {
            Some(result) => {
                if let Some(val) = result.get_key_duration(Utc::now().timestamp()) {
                    if val.len() > MAX_BUFFER_SIZE {
                        let mut response = "OK\r\ntransfer-encoding: chunked\r\n\r\n".to_string();

                        let num_chunks = val.len() / MAX_BUFFER_SIZE;
                        let remainder = val.len() % MAX_BUFFER_SIZE;

                        //loop  chunk
                        for i in 0..num_chunks {
                            let start = i * MAX_BUFFER_SIZE;
                            let end = start + MAX_BUFFER_SIZE;
                            let chunk = &val[start..end];

                            response.push_str(&format!("{}\r\n", MAX_BUFFER_SIZE));
                            response.push_str(chunk);
                            response.push_str("\r\n");
                        }

                        if remainder > 0 {
                            let start = num_chunks * MAX_BUFFER_SIZE;
                            let chunk = &val[start..];

                            response.push_str(&format!("{}\r\n", remainder));
                            response.push_str(chunk);
                            response.push_str("\r\n");
                        }

                        // final chunk
                        response.push_str("0\r\n\r\n");

                        response
                    } else {
                        "OK\r\n\r\n".to_string() + &val + "\r\n\r\n"
                    }
                } else {
                    self.data.remove(key);
                    return "Err\r\n".to_string();
                }
            }
            None => {
                return "OK\r\n\r\n".to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTcpStream {
        read_data: Vec<u8>,
        position: usize,
    }

    impl MockTcpStream {
        fn new(data: Vec<u8>) -> Self {
            MockTcpStream {
                read_data: data,
                position: 0,
            }
        }
    }
    impl AsyncRead for MockTcpStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let remaining = &self.read_data[self.position..];
            let to_read = std::cmp::min(remaining.len(), buf.remaining());

            if to_read == 0 {
                return Poll::Ready(Ok(()));
            }

            buf.put_slice(&remaining[..to_read]);
            self.position += to_read;

            Poll::Ready(Ok(()))
        }
    }

    #[test]
    fn test_receive_message_ok_blank() {
        let mut share_memory = ShareMemory::new();
        let message = "set key1\r\n\r\nvalue1\r\n\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n\r\n");

        let message = "get key1".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue1\r\n\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\n");
    }

    #[test]
    fn test_receive_message_ok_set_error() {
        let mut share_memory = ShareMemory::new();
        let message = "set key1\r\nvalue1\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n\r\n");
    }

    #[test]
    fn test_receive_message_ok_value() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\n\r\nvalue2\r\n\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n\r\n");
    }

    #[test]
    fn test_receive_message_error_empty_text() {
        let mut share_memory = ShareMemory::new();
        let message = "".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n\r\n");

        let mut share_memory = ShareMemory::new();
        let message = "".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n\r\n");
    }
    #[test]
    fn test_receive_message_error_wrong_format() {
        let mut share_memory = ShareMemory::new();
        let message = "sexxxxxx1".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n\r\n");

        let mut share_memory = ShareMemory::new();
        let message = "".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n\r\n");
    }

    #[test]
    fn test_set_duration_success() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\nduration: 300\r\n\r\nvalue2\r\n\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n\r\n");
    }

    #[test]
    fn test_set_duration_success_1sec() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\nduration: 1\r\n\r\nvalue2\r\n\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n\r\n");
    }

    #[test]
    fn test_set_duration_success_1second() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\nduration: 1\r\n\r\nvalue2\r\n\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n\r\n");
    }

    #[test]
    fn test_set_duration() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\n\r\nvalue2\r\n\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n\r\n");
    }

    #[test]
    fn test_get_key_duration_success() {
        let duration = 100;
        let test_text = "".to_string();
        let obj_mem = ObjectMemory {
            duration_sec: duration,
            raw_data: test_text.clone(),
            created_at: Utc::now().timestamp(),
        };
        let curr = Utc::now().timestamp();
        if let Some(value) = obj_mem.get_key_duration(curr) {
            assert_eq!(value, test_text);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_get_key_duration_success_duration() {
        let duration = 100;
        let test_text = "".to_string();
        let obj_mem = ObjectMemory {
            duration_sec: duration,
            raw_data: test_text.clone(),
            created_at: Utc::now().timestamp(),
        };
        let curr = Utc::now().timestamp() + duration;
        if let Some(value) = obj_mem.get_key_duration(curr) {
            assert_eq!(value, test_text);
        } else {
            assert!(false);
        }
    }
    #[test]
    fn test_get_key_duration_fails_sec() {
        let duration = 100;
        let test_text = "".to_string();
        let obj_mem = ObjectMemory {
            duration_sec: duration,
            raw_data: test_text.clone(),
            created_at: Utc::now().timestamp(),
        };
        let curr = Utc::now().timestamp() + duration + 20;
        if let Some(_value) = obj_mem.get_key_duration(curr) {
            assert!(false);
        } else {
            assert!(true);
        }
    }

    #[tokio::test]
    async fn test_recv_data_raw_normal() {
        let mut share_memory = ShareMemory::new();

        let v = "value2";
        let message = format!("set key2\r\n\r\n{}\r\n\r\n", v);

        let mut mock_stream = MockTcpStream::new(message.into());
        let result = share_memory.recv_data_raw(&mut mock_stream).await;
        let obj_mem = result.unwrap();
        assert_eq!(obj_mem.raw_data, v);
    }

    #[tokio::test]
    async fn test_recv_data_raw_chunked() {
        let mut share_memory = ShareMemory::new();

        let num1 = 1000;
        let num2 = 6000;
        let data1 = "a".repeat(num1);
        let data2 = "b".repeat(num2);

        let set_data = format!(
            "set test_key\r\ntransfer-encoding: chunked\r\n\r\n{:X}\r\n{}\r\n{:X}\r\n{}\r\n0\r\n\r\n",
            data1.len(),
            data1,
            data2.len(),
            data2
        );

        let mut mock_stream = MockTcpStream::new(set_data.into());
        let result = share_memory.recv_data_raw(&mut mock_stream).await;
        let obj_mem = result.unwrap();
        assert_eq!(obj_mem.raw_data.len(), num1 + num2);
    }
    #[tokio::test]
    async fn test_recv_n_get_data() {
        let mut share_memory = ShareMemory::new();
        let test_data = "a".repeat(1000);
        let obj_mem = ObjectMemory {
            duration_sec: 300,
            raw_data: test_data.clone(),
            created_at: Utc::now().timestamp(),
        };
        share_memory
            .data
            .insert("test_large_buffer".to_string(), obj_mem);

        let response = share_memory.get_data("test_large_buffer");

        assert_eq!(response, format!("OK\r\n\r\n{}\r\n\r\n", test_data));
    }
    #[tokio::test]
    async fn test_recv_n_get_data_chunked() {
        let mut share_memory = ShareMemory::new();
        let txt_b = "b".repeat(100);
        let txt_a = "a".repeat(MAX_BUFFER_SIZE);
        let test_data = txt_a.clone() + &txt_b;

        let obj_mem = ObjectMemory {
            duration_sec: 300,
            raw_data: test_data,
            created_at: Utc::now().timestamp(),
        };
        share_memory
            .data
            .insert("test_large_buffer".to_string(), obj_mem);

        let response = share_memory.get_data("test_large_buffer");

        assert_eq!(
            response,
            format!(
                "OK\r\ntransfer-encoding: chunked\r\n\r\n{}\r\n{}\r\n{}\r\n{}\r\n0\r\n\r\n",
                MAX_BUFFER_SIZE,
                txt_a,
                txt_b.len(),
                txt_b
            )
        );
    }

    #[tokio::test]
    async fn test_recv_n_get_data_multiple_chunks() {
        let mut share_memory = ShareMemory::new();
        let test_data = "a".repeat(4 * MAX_BUFFER_SIZE + 100);

        let obj_mem = ObjectMemory {
            duration_sec: 300,
            raw_data: test_data.clone(),
            created_at: Utc::now().timestamp(),
        };
        share_memory
            .data
            .insert("test_multiple_chunks".to_string(), obj_mem);

        let response = share_memory.get_data("test_multiple_chunks");
        let mut expected_response = "OK\r\ntransfer-encoding: chunked\r\n\r\n".to_string();

        for _ in 0..4 {
            expected_response.push_str(&format!("{}\r\n", MAX_BUFFER_SIZE));
            expected_response.push_str(&"a".repeat(MAX_BUFFER_SIZE));
            expected_response.push_str("\r\n");
        }

        expected_response.push_str(&format!("100\r\n"));
        expected_response.push_str(&"a".repeat(100));
        expected_response.push_str("\r\n");
        expected_response.push_str("0\r\n\r\n");

        assert_eq!(response, expected_response);
    }
}
