pub struct ObjectMemory {
    pub raw_data: String,
    pub duration_sec: i64,
    pub created_at: i64,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make(duration: i64) -> ObjectMemory {
        ObjectMemory {
            duration_sec: duration,
            raw_data: "".to_string(),
            created_at: Utc::now().timestamp(),
        }
    }

    #[test]
    fn test_get_key_duration_success() {
        let obj_mem = make(100);
        let curr = Utc::now().timestamp();
        assert_eq!(obj_mem.get_key_duration(curr), Some("".to_string()));
    }

    #[test]
    fn test_get_key_duration_success_duration() {
        let obj_mem = make(100);
        let curr = Utc::now().timestamp() + 100;
        assert_eq!(obj_mem.get_key_duration(curr), Some("".to_string()));
    }

    #[test]
    fn test_get_key_duration_fails_sec() {
        let obj_mem = make(100);
        let curr = Utc::now().timestamp() + 100 + 20;
        assert_eq!(obj_mem.get_key_duration(curr), None);
    }
}
