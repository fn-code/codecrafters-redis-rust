use anyhow::Error;
use bytes::BytesMut;
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

                    let rdb_str = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
                    let rdb = hex::decode(rdb_str);

                    match rdb {
                        Ok(rdb) => {
                            println!("--------------- sending RDB len: {}", rdb.len());
                            let msg =  format!("${}\r\n", rdb.len());
                            let mut message_bytes = BytesMut::from(msg.as_bytes());
                            message_bytes.extend_from_slice(&rdb);

                            conn.write(&message_bytes).await?;
                            println!("--------------- sent RDB");

                        }
                        Err(e) => {
                            eprintln!("Failed to decode RDB: {:?}", e);
                        }
                    }
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