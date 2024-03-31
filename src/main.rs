// Uncomment this block to pass the first stage

mod resp;
mod storage;


use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use crate::resp::Value;
use anyhow::Result;
use crate::storage::Database;





#[tokio::main]
async fn main() {

    let db = Arc::new(storage::Database::new());
    let mut host = String::from("0.0.0.0");
    let mut port = 6379;

    let mut args = std::env::args().into_iter();



    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => {
                let host_args = args.next().expect("missing host argument");
                host = host_args;

            }

            "--port" => {
                port = args
                    .next()
                    .expect("missing port argument")
                    .parse::<u16>()
                    .expect("invalid port argument");
            }
            _ => ()
        }
    }


    println!("Starting server on {host}:{port}");
    run_server(db, host, port).await;
}

async fn run_server(db: Arc<Database>, host: String, port: u16) {
    println!("Trying Listening on {host}:{port}");
    let listener = TcpListener::bind(format!("{host}:{port}")).await.unwrap();
    loop {
        let stream = listener.accept().await;
        let db = db.clone();
        match stream {
            Ok((stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {
                    handle_conn(stream, db).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

// *2\r\n$4\r\necho\r\n$3\r\nhey\r\n
async fn handle_conn(stream: TcpStream, db: Arc<Database>) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();
        println!("Got value: {:?}", value);

        let response = if let Some(value) = value {
            let (command, args) = extract_command(value).unwrap();
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "get" => {
                    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                    match db.get(key.as_str()) {
                        Some(value) => Value::BulkString(value),
                        None => Value::NullBulkString
                    }
                }
                "set" => {
                    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                    let value = unpack_bulk_str(args.get(1).unwrap().clone()).unwrap();


                    match args.get(2) {
                        Some(Value::BulkString(sub_cmd)) if sub_cmd.as_str().to_ascii_lowercase() == "px" => {
                            if let None = args.get(3) {
                                println!("not got expiration");
                                Value::NullBulkString
                            } else {
                                let expiration = match args.get(3).unwrap() {
                                    Value::BulkString(s) => s.parse::<i64>().unwrap(),
                                    _ => panic!("Invalid expiration")
                                };

                                db.set(key, value, Some(expiration));
                                Value::SimpleString("OK".to_string())
                            }

                        },
                        _ => {
                           db.set(key, value, None);
                            Value::SimpleString("OK".to_string())
                        }

                    }


                }
                c => panic!("Unsupported command: {}", c)
            }
        } else {
            break;
        };

        println!("Sending value {:?}", response);
        handler.write_value(response).await.unwrap();
    }

}


fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => {
            Ok((
                unpack_bulk_str(a.first().unwrap().clone())?,
                a.into_iter().skip(1).collect(),
                ))
        }
        _ => Err(anyhow::anyhow!("Not an array"))
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Not a bulk string"))
    }
}