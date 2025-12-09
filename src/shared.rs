use chrono::Utc;
use std::collections::HashMap;

use std::env;

pub struct ObjectMemory {
    pub txt: String,
    pub duration_sec: i64,
    pub created_at: i64,
}

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

    pub fn receive_message(&mut self, message: String) -> String {
        let parts: Vec<&str> = message.split("\r\n").collect();
        let header = parts[0];
        let header_message: Vec<&str> = header.split(' ').collect();

        if header_message.len() >= 2 {
            let method_name = header_message[0].to_string().to_lowercase();
            if method_name == "set" {
                let key_data = header_message[1].to_string();
                if parts.len() <= 2 || !parts[1].is_empty() {
                    return "Err\r\n".to_string();
                }

                let value = parts.get(2).unwrap_or(&"").to_string();

                let expire_timeout;
                if header_message.len() == 3 {
                    let expire_timeout_str = header_message[2].to_string();
                    if let Ok(result) = expire_timeout_str.parse::<i64>() {
                        expire_timeout = result;
                    } else {
                        expire_timeout = env::var("EXPIRE_TIMEOUT")
                            .unwrap_or("300".to_string())
                            .parse::<i64>()
                            .unwrap_or(300);
                    }
                } else {
                    // 60 seconds * 5 minute = 300 seconds
                    expire_timeout = env::var("EXPIRE_TIMEOUT")
                        .unwrap_or("300".to_string())
                        .parse::<i64>()
                        .unwrap_or(300);
                }

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
