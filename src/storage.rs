
use tokio::time::Instant;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Item {
    value: String,
    created_at: Instant,
    expiration: Option<i64>,
}

pub struct Database {
    db: Mutex<HashMap<String, Item>>
}

impl Database {
    pub fn new() -> Self {
        Database {
            db: Mutex::new(HashMap::new())
        }
    }

    pub fn set(&self, key: String, value: String, expiration: Option<i64>) {
        let mut db = self.db.lock().unwrap();
        db.insert(key, Item {
            value,
            created_at: Instant::now(),
            expiration
        });
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let db = self.db.lock().unwrap();
        if let Some(item) = db.get(key) {
           if let Some(expaiation) = item.expiration {
               if item.created_at.elapsed().as_millis() > expaiation as u128 {
                   return None;
               }
           }
            return Some(item.value.clone());
        }
        None
    }
}