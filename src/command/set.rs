use anyhow::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::conn::Connnection;
use crate::storage::Storage;
use crate::value::Value;

#[derive(Debug)]
pub(crate) struct Set {
    key: String,
    value: String,
    unit: String,
    expire: i64,
}

impl Set {
    pub(crate) fn new(args: Vec<Value>) -> Option<Self> {

        if args.len() < 4 {
            return None;
        }

        let key = args.get(0).cloned().unwrap_or(Value::NullBulkString).to_string();
        let value = args.get(1).cloned().unwrap_or(Value::NullBulkString).to_string();
        let unit = args.get(2).cloned().unwrap_or(Value::NullBulkString).to_string();


        let expire = match args.get(3).cloned() {
            Some(Value::SimpleString(s)) => s.parse::<i64>().unwrap_or(0),
            _ => 0
        };


        Some(Set {
            key,
            value,
            unit,
            expire,
        })
    }


    pub(crate) async fn execute<D>(&self, conn: &mut Connnection, db: Arc<Mutex<D>>) -> Result<(), Error>
    where D: Storage {

        if self.key.is_empty() {
            return Err(anyhow::anyhow!("Key cannot be empty"));
        }

        if self.unit.is_empty() {
            return Err(anyhow::anyhow!("Unit cannot be empty"));
        }

        if self.expire == 0 {
            return Err(anyhow::anyhow!("Expire cannot be empty"));
        }

        db.lock().await.set(
            self.key.clone(),
            self.value.clone(),
            Some(self.expire),
        );

        conn.write_value(
            Value::SimpleString("OK".to_string())
        ).await?;
        Ok(())
    }
}