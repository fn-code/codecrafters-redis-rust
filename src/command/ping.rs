use crate::conn::Connnection;
use anyhow::Error;
use crate::value::Value;

#[derive(Debug)]
pub(crate) struct Ping  {

}

impl Ping {
    pub(crate) fn new() -> Self {
        Ping {}
    }

    pub(crate) async fn execute(&self, conn: &mut Connnection) -> Result<(), Error> {
        let response = Value::SimpleString("PONG".to_string());
        conn.write_value(response).await?;
        Ok(())
    }
}