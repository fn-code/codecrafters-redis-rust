// Uncomment this block to pass the first stage

use std::sync::Arc;
use tokio::sync::Mutex;
use redis_starter_rust::{config::Config, storage::Database, server::Server, conn::Connnection};


use anyhow::Error;

#[tokio::main]
async fn main() ->  Result<(), Error>{

    let config = Config::parse();
    let db = Arc::new(Mutex::new(Database::new()));
    let server = Arc::new(Server::new(config, db));

    if let Ok(stream) = server.connect_to_master().await {
        let conn = Connnection::new(stream);

       server.handshake(conn).await?;
    }

    let listener = server.listen().await?;

    while let Ok((stream, _)) = listener.accept().await {
        let conn = Connnection::new(stream);
        let server = Arc::clone(&server);

        tokio::spawn(async move {
           if let Err(err) = server.handle_connection(conn).await {
               eprintln!("Error: {err}");
           }
        });
    }

    Ok(())

}

