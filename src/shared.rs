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
