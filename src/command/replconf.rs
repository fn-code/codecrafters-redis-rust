use anyhow::Error;
use crate::conn::Connnection;
use crate::value::Value;

#[derive(Debug)]
pub(crate) struct Replconf {
    conf: Vec<Value>,
}

impl Replconf {
    pub(crate) fn new(conf: Vec<Value>) -> Self {
         Replconf {
            conf
        }
    }

    pub(crate) async fn execute(&self, conn: &mut Connnection) -> Result<(), Error> {
        let value = Value::SimpleString(String::from("OK"));

        conn.write_value(value).await?;
        Ok(())
    }

    pub(crate) async fn send(&self, conn: &mut Connnection) -> Result<(), Error> {
        let mut values = vec![
            Value::BulkString(String::from("REPLCONF")),
        ];

        for v in self.conf.clone() {
            values.push(v);
        }

        let repl_value = Value::Array(values);


        conn.write_value(repl_value).await?;
        Ok(())
    }
}