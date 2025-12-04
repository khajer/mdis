use std::collections::HashMap;

pub struct ShareMemory {
    pub data: HashMap<String, String>,
}
impl ShareMemory {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn receive_message(&mut self, message: String) -> String {
        let parts: Vec<&str> = message.split('\n').collect();
        let header = parts[0];
        let header_message: Vec<&str> = header.split(' ').collect();

        if header_message.len() == 2 {
            let method_name = header_message[0].to_string().to_lowercase();
            if method_name == "set" {
                let key_data = header_message[1].to_string();
                let value = parts[1].to_string();
                self.data.insert(key_data, value);

                return "OK\ninsert completed\r\n".to_string();
            } else if method_name == "get" {
                let key_data = header_message[1].to_string();
                let result = self
                    .data
                    .get(&key_data)
                    .cloned()
                    .unwrap_or_else(|| "".to_string());

                return "OK\n".to_string() + &result + "\r\n";
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
        let message = "set key1\nvalue1".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\ninsert completed\r\n");

        let message = "get key1".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\nvalue1\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\n\r\n");
    }

    #[test]
    fn test_receive_message_ok_value() {
        let mut share_memory = ShareMemory::new();
        let message = "set key2\nvalue2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\ninsert completed\r\n");

        let message = "get key2".to_string();
        let response = share_memory.receive_message(message);
        assert_eq!(response, "OK\nvalue2\r\n");
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
}
