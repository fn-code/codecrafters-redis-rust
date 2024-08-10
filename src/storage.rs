
use tokio::time::Instant;
use std::collections::HashMap;

pub trait Storage {
    fn set(&mut self, key: String, value: String, expiration: Option<i64>);
    fn get(&mut self, key: &str) -> Option<String>;
}

pub struct Item {
    value: String,
    created_at: Instant,
    expiration: Option<i64>,
}

pub struct Database {
    db: HashMap<String, Item>
}

impl Database {
    pub fn new() -> Self {
        Database {
            db: HashMap::new()
        }
    }
}

impl Storage for Database {
    fn set(&mut self, key: String, value: String, expiration: Option<i64>) {
        self.db.insert(key, Item {
            value,
            created_at: Instant::now(),
            expiration
        });
    }

    fn get(&mut self, key: &str) -> Option<String> {
        if let Some(item) = self.db.get(key) {
           if let Some(expairation) = item.expiration {
               let now = Instant::now();
               if now.duration_since(item.created_at).as_millis() > expairation as u128 {
                   return None;
               }
           }
            return Some(item.value.clone());
        }
        None
    }
}