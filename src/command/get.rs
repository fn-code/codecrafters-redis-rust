use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Error;
use crate::conn::Connnection;
use crate::storage::Storage;
use crate::value::Value;

#[derive(Debug)]
pub(crate) struct Get {
    key: String
}

impl Get {
    pub(crate) fn new(args: Vec<Value>) -> Option<Self> {

        if args.len() != 1 {
            return None;
        }

        args.get(0).cloned().map(|v| Get { key: v.to_string() })
    }

    pub(crate) async fn execute<D>(
        &self,
        conn: &mut Connnection,
        db: Arc<Mutex<D>>
    ) -> Result<(), Error>
    where D: Storage {

        let response = if let Some(val) = db.lock().await.get(&self.key) {
            Value::BulkString(val)
        } else {
            Value::NullBulkString
        };

        conn.write_value(response).await?;
        Ok(())
    }
}