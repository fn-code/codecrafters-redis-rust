use anyhow::Error;
use crate::config::Config;
use crate::replication::{Replication, Role};
use crate::conn::Connnection;
use crate::value::Value;

pub(crate) struct Info;


impl Info {
    pub(crate) fn new() -> Self {
        Info
    }
    pub(crate) async fn execute(&self,
                   conn: &mut Connnection,
                   _config: &Config,
                   repl: &Replication
    ) -> Result<(), Error> {

        let s = match repl.role {
            Role::Master => {
                let info = [
                    format!("role:{}", repl.role),
                    format!("master_replid:{}", repl.master_replid),
                    format!("master_repl_offset:{}", repl.master_repl_offset),
                ];

                info.join("\n")
            }
            Role::Slave => format!("role:{}", repl.role),
        };

        let value = Value::BulkString(s);
        conn.write_value(value).await?;

        Ok(())
    }
}