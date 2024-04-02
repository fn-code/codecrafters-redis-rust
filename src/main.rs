// Uncomment this block to pass the first stage

mod resp;
mod storage;


use std::cell::RefCell;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use crate::resp::Value;
use anyhow::Result;
use crate::storage::Database;


pub enum ServerRole {
    Master,
    Slave
}
impl ServerRole {
    pub fn to_string(&self) -> String {
        match self {
            ServerRole::Master => "master".to_string(),
            ServerRole::Slave => "slave".to_string()
        }
    }
}

pub struct Server {
    pub role: ServerRole,
    pub port: u16,
    pub host: String,

}

#[tokio::main]
async fn main() {

    let db = Arc::new(storage::Database::new());

    let server = Arc::new(RefCell::new(Server {
        role: ServerRole::Master,
        port: 6379,
        host:  String::from("0.0.0.0"),
    }));



    let mut args = std::env::args().into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => {
                let host_args = args.next().expect("missing host argument");
                server.borrow_mut().host = host_args;
            }

            "--port" => {
                server.borrow_mut().port = args
                    .next()
                    .expect("missing port argument")
                    .parse::<u16>()
                    .expect("invalid port argument");
            }
            _ => ()
        }
    }



    println!("Starting server on {}:{}", server.borrow().host, server.borrow().port);

    run_server(&db, &server).await;
}

async fn run_server(db: &Arc<Database>, srv: &Arc<RefCell<Server>>) {

    println!("Trying Listening on {}:{}", srv.borrow().host, srv.borrow().port);
    let listener = TcpListener::bind(format!("{}:{}", srv.borrow().host, srv.borrow().port)).await.unwrap();
    loop {
        let stream = listener.accept().await;
        let db = Arc::clone(db);
        // let _server = Arc::clone(srv);
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
                "info" => {

                    if let Some(command_value) = args.first() {
                        let command_str = unpack_bulk_str(command_value.clone()).unwrap();

                        if command_str.to_lowercase() == "replication" {
                            Value::BulkString(format!("role:{}", "master"))
                        } else {
                            Value::NullBulkString
                        }

                    } else {
                        Value::NullBulkString
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