use anyhow::Error;

use crate::{
    conn::Connnection,
    value::Value,
    replication::Replication,
};

#[derive(Debug)]
pub(crate) struct Psync {
    args: Vec<Value>,
}

impl Psync {
    pub(crate) fn new(args: Vec<Value>) -> Self {
        Psync {
            args
        }
    }

    pub(crate) async fn execute(&self, conn: &mut Connnection, repl: &Replication) -> Result<(), Error> {
        match (self.args.get(0), self.args.get(1)) {
            (Some(a), Some(b)) => {
                if a.get_string() == Some(String::from("?")) && b.get_string() == Some(String::from("-1")) {
                    let resp_value = Value::SimpleString(format!("FULLRESYNC {} 0", repl.master_replid));
                    conn.write_value(resp_value).await?;
                }
            }
            _ => {}
        }

        Ok(())

    }

    pub(crate) async fn send(&self, conn: &mut Connnection) -> Result<(), Error> {
        let mut values = vec![
            Value::BulkString(String::from("PSYNC")),
        ];

        self.args.iter().for_each(|v| {
            values.push(v.clone());
        });

        conn.write_value(Value::Array(values)).await?;
        Ok(())
    }
}