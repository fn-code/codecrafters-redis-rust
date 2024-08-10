use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Error;
use tokio::net::{TcpListener, TcpStream};
use crate::conn::Connnection;
use anyhow::Result;

use crate::{
    command::Command,
    command::psync::Psync,
    command::replconf::Replconf,
    command::ping::Ping,
    value::Value,
    replication::Replication,
    config::Config
};
use crate::storage::Storage;

pub struct Server<D>
where
    D: Storage
{
    replica: Replication,
    config: Config,
    db: Arc<Mutex<D>>,
}

impl<D> Server<D>
where
    D: Storage
{
    pub fn new(config: Config,  db: Arc<Mutex<D>>) -> Self {
        Server {
            replica: Replication::new(&config),
            config,
            db,
        }
    }


    pub async fn listen(&self) -> Result<TcpListener, Error> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.config.port)).await?;
        Ok(listener)
    }

    pub async fn connect_to_master(&self) -> Result<TcpStream, Error> {
        if let Some(replicaof) = self.config.replicaof.clone() {
            let stream = TcpStream::connect(replicaof).await?;
            return Ok(stream)
        }

        Err(Error::msg("No master to connect to"))
    }

    pub async fn handle_connection(&self, mut conn: Connnection) -> Result<(), Error> {

        loop {
            let Ok(value) = conn.read_value().await else {
                break Err(Error::msg("Failed to read value"));
            };

            println!("Received value {:?}", value);

            let cmd = Command::parse(value);

            match cmd {
                Command::Ping(ping) => {
                    ping.execute(&mut conn).await?;
                },
                Command::Echo(echo) => {
                    echo.execute(&mut conn).await?;
                },
                Command::Get(get) => {
                    let db = Arc::clone(&self.db);
                    get.execute(&mut conn, db).await?;
                },
                Command::Info(info) => {
                    info.execute(&mut conn, &self.config, &self.replica).await?;
                },
                Command::Set(set) => {
                    let db = Arc::clone(&self.db);
                    set.execute(&mut conn, db).await?;
                },
                Command::Replconf(replconf) => {
                    replconf.execute(&mut conn).await?;
                },
                Command::Psync(psync) => {
                    psync.execute(&mut conn, &self.replica).await?;
                },
                _ => {}
            }

        }
    }


    pub async fn handshake(&self, mut conn: Connnection) -> Result<(), Error> {
        Ping::new().execute(&mut conn).await?;

        let _value = conn.read_value().await?;

        Replconf::new(vec![Value::BulkString(String::from("listening-port")), Value::BulkString(String::from("6380"))])
            .send(&mut conn)
            .await?;


        let _value = conn.read_value().await?;

        Replconf::new(vec![Value::BulkString(String::from("capa")), Value::BulkString(String::from("psync2"))])
            .send(&mut conn)
            .await?;

        let _value = conn.read_value().await?;

        Psync::new(vec![Value::BulkString(String::from("?")), Value::BulkString(String::from("-1"))])
            .send(&mut conn)
            .await?;

        let _value = conn.read_value().await?;

        Ok(())
    }

}