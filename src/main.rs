// Uncomment this block to pass the first stage

mod resp;
mod storage;


use std::cmp::PartialEq;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use crate::resp::Value;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use crate::storage::Database;


#[derive(PartialEq)]
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
    pub master_host: String,
    pub master_port: u16,
    pub master_replid: String,
    pub master_repl_offset: usize,

}

#[tokio::main]
async fn main() {

    let db = Arc::new(storage::Database::new());

    let server = Arc::new(RwLock::new(Server {
        role: ServerRole::Master,
        port: 6379,
        host:  String::from("0.0.0.0"),
        master_host: "".to_string(),
        master_port: 0,
        master_replid: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
        master_repl_offset: 0
    }));

    let mut args = std::env::args().into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => {
                let host_args = args.next().expect("missing host argument");
                let mut srv_write = server.write().unwrap();
                srv_write.host = host_args;
            }

            "--port" => {

                let mut srv_write = server.write().unwrap();
                srv_write.port = args
                    .next()
                    .expect("missing port argument")
                    .parse::<u16>()
                    .expect("invalid port argument");
            }
            "--replicaof" => {
                let master_host = args.next().expect("missing master host argument");
                let master_port = args
                    .next()
                    .expect("missing master port argument")
                    .parse::<u16>()
                    .expect("invalid master port argument");


                let mut srv_write = server.write().unwrap();
                srv_write.role = ServerRole::Slave;
                srv_write.master_host = master_host;
                srv_write.master_port = master_port;

            }
            _ => ()
        }
    }

    

    let srv_read = server.read().unwrap();
    println!("Starting server on {}:{}", srv_read.host, srv_read.port);
    run_server(&db, &server).await;
    
}

async fn connect_to_master(srv: &Arc<RwLock<Server>>)  {

    let srv_read = srv.read().unwrap();
    println!("Connecting to master at {}:{}", srv_read.master_host,srv_read.master_port);
    let ip4_addr = SocketAddr::from_str(format!("{}:{}",srv_read.master_host,srv_read.master_port).as_str()).unwrap();
    let client = TcpSocket::new_v4().unwrap();
    let mut stream = client.connect(ip4_addr).await.unwrap();

    stream.write(Value::Array(vec![
        Value::BulkString("ping".to_string()),
    ]).serialize().as_bytes()).await.unwrap();

}



async fn run_server(db: &Arc<Database>, srv: &Arc<RwLock<Server>>) {

    let srv_read = srv.read().unwrap();


    // if the app role is slave, connect to the master
    if srv_read.role == ServerRole::Slave {
        println!("Trying to connect to master node {}:{}", srv_read.master_host, srv_read.master_port);
        connect_to_master(srv).await;
    }

    println!("Trying Listening on {}:{}",srv_read.host, srv_read.port);
    let listener = TcpListener::bind(format!("{}:{}",srv_read.host, srv_read.port)).await.unwrap();
    loop {
        let stream = listener.accept().await;
        let db = Arc::clone(db);
        let srv = Arc::clone(srv);
        match stream {
            Ok((stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {
                    handle_conn(stream, &db, &srv).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

// *2\r\n$4\r\necho\r\n$3\r\nhey\r\n
async fn handle_conn(stream: TcpStream, db: &Arc<Database>, srv: &Arc<RwLock<Server>>) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();
        println!("Got value: {:?}", value);

        let response = if let Some(value) = value {
            let (command, args) = extract_command(value).unwrap();
            match command.to_lowercase().as_str() {
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

                            Value::BulkString(format!("role:{}\nmaster_replid:{}\nmaster_repl_offset:{}",
                                                      srv.read().unwrap().role.to_string(),
                                                      srv.read().unwrap().master_replid,
                                                      srv.read().unwrap().master_repl_offset
                            ))
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