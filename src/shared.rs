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
            let key = header_message[0].to_string().to_lowercase();
            if key == "set" {
                let value = header_message[1].to_string();
                self.data.insert(key, value);
                return "Ok\ninsert completed\r\n".to_string();
            } else if key == "get" {
                let result = self
                    .data
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| "".to_string());

                return "Ok".to_string() + &result + "\r\n";
            } else {
                return "Err\r\n".to_string();
            }
        }

        "Err\r\n".to_string()
    }
}
