use chrono::Utc;
use std::collections::HashMap;
use std::env;
use tokio::io::{AsyncReadExt, Error};
use tokio::net::TcpStream;
pub struct ObjectMemory {
    pub txt: String,
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
            Some(self.txt.clone())
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

    pub fn receive_message(&mut self, message: String) -> String {
        if message.contains("\r\n\r\n") {
            let message_parts: Vec<&str> = message.split("\r\n\r\n").collect();
            if message_parts.len() != 2 {
                return "Err\r\n".to_string();
            }

            let header_part = message_parts[0];
            let data_part = message_parts[1];

            let header_lines: Vec<&str> = header_part.split("\r\n").collect();
            if header_lines.is_empty() {
                return "Err\r\n".to_string();
            }

            let first_line_parts: Vec<&str> = header_lines[0].split(' ').collect();
            if first_line_parts.len() < 2 {
                return "Err\r\n".to_string();
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
                        txt: trimmed_data.to_string(),
                        duration_sec: expire_timeout,
                        created_at: Utc::now().timestamp(),
                    },
                );

                return "OK\r\ninsert completed\r\n".to_string();
            } else if method_name == "get" {
                match self.data.get(&key_data) {
                    Some(result) => {
                        if let Some(v) = result.get_key_duration(Utc::now().timestamp()) {
                            return "OK\r\n\r\n".to_string() + &v + "\r\n";
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
                return "Err\r\n".to_string();
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
                            return "Err\r\n".to_string();
                        }
                    }

                    if parts.len() <= value_line || !parts[value_line - 1].is_empty() {
                        return "Err\r\n".to_string();
                    }

                    let value = parts.get(value_line).unwrap_or(&"").to_string();

                    self.data.insert(
                        key_data,
                        ObjectMemory {
                            txt: value,
                            duration_sec: expire_timeout,
                            created_at: Utc::now().timestamp(),
                        },
                    );

                    return "OK\r\ninsert completed\r\n".to_string();
                } else if method_name == "get" {
                    let key_data = header_message[1].to_string();
                    match self.data.get(&key_data) {
                        Some(result) => {
                            if let Some(v) = result.get_key_duration(Utc::now().timestamp()) {
                                return "OK\r\n\r\n".to_string() + &v + "\r\n";
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
                    return "Err\r\n".to_string();
                }
            }

            "Err\r\n".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receive_message_ok_blank() {
        let mut share_memory = ShareMemory::new();
        let message = "set key1\r\n\r\nvalue1\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n");

        let message = "get key1".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue1\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\n");
    }

    #[test]
    fn test_receive_message_ok_set_error() {
        let mut share_memory = ShareMemory::new();
        let message = "set key1\r\nvalue1\r\n".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n");
    }

    #[test]
    fn test_receive_message_ok_value() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\n\r\nvalue2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n");
    }

    #[test]
    fn test_receive_message_error_empty_text() {
        let mut share_memory = ShareMemory::new();
        let message = "".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n");

        let mut share_memory = ShareMemory::new();
        let message = "".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n");
    }
    #[test]
    fn test_receive_message_error_wrong_format() {
        let mut share_memory = ShareMemory::new();
        let message = "sexxxxxx1".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n");

        let mut share_memory = ShareMemory::new();
        let message = "".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "Err\r\n");
    }

    #[test]
    fn test_set_duration_success() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\nduration: 300\r\n\r\nvalue2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n");
    }

    #[test]
    fn test_set_duration_success_1sec() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\nduration: 1\r\n\r\nvalue2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n");
    }

    #[test]
    fn test_set_duration_success_1second() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\nduration: 1\r\n\r\nvalue2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n");
    }

    #[test]
    fn test_set_duration() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\r\n\r\nvalue2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\ninsert completed\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\r\n\r\nvalue2\r\n");
    }

    #[test]
    fn test_get_key_duration_success() {
        let duration = 100;
        let test_text = "".to_string();
        let obj_mem = ObjectMemory {
            duration_sec: duration,
            txt: test_text.clone(),
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
            txt: test_text.clone(),
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
            txt: test_text.clone(),
            created_at: Utc::now().timestamp(),
        };
        let curr = Utc::now().timestamp() + duration + 20;
        if let Some(_value) = obj_mem.get_key_duration(curr) {
            assert!(false);
        } else {
            assert!(true);
        }
    }
}
