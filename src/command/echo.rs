use anyhow::Error;
use crate::conn::Connnection;
use crate::value::Value;

#[derive(Debug)]
pub(crate) struct Echo  {
    msg:  Vec<Value>
}

impl Echo {
    pub fn new(msg: Vec<Value>) -> Self {
        Echo {
            msg
        }
    }

    pub(crate) async fn execute(&self, conn: &mut Connnection) -> Result<(), Error> {
        // get the first message
        let response = self.msg.first().unwrap().clone();

        conn.write_value(response).await?;
        Ok(())
    }
}