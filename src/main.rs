// Uncomment this block to pass the first stage

mod resp;
mod storage;

use std::cmp::PartialEq;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use crate::resp::{Value};
use anyhow::{Result};
use crate::storage::Database;
use std::{time};
use tokio::time::timeout;


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
                if srv_write.master_host == "localhost" {
                    srv_write.master_host = "127.0.0.1".to_string();
                }
                srv_write.master_port = master_port;

            }
            _ => ()
        }
    }

    

    let srv_read = server.read().unwrap();
    println!("Starting server on {}:{}", srv_read.host, srv_read.port);
    run_server(&db, &server).await;
    
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


async fn connect_to_master(server: &Arc<RwLock<Server>>)  {

    let srv_read = server.read().unwrap();
    println!("Connecting to master at {}:{}", srv_read.master_host,srv_read.master_port);
    let ip4_addr = SocketAddr::from_str(format!("{}:{}",srv_read.master_host,srv_read.master_port).as_str()).unwrap();
    let stream  = TcpStream::connect(ip4_addr).await.unwrap();



    handle_slave_con(stream, &server).await;
}

async fn handle_slave_con(stream: TcpStream, server: &Arc<RwLock<Server>>) {
    let srv_read = server.read().unwrap();
    let mut handler = resp::RespHandler::new(stream);



    handler.write_value(Value::Array(vec![
        Value::BulkString("ping".to_string()),
    ])).await.unwrap();

    // let resp = timeout(time::Duration::from_secs(10), handler.read_value()).await.unwrap().unwrap();
    let resp = handler.read_value().await.unwrap();


    match resp {
        Some(value) => {
            let (command, _ ) = extract_command(value).unwrap();
            if command.to_lowercase() != "pong" {
                println!("Slave did not receive pong from master");
                return;
            }

            println!("Slave received pong from master")
        }
        None => {
            println!("Slave received null value, should got pong");
            return;
        }
    };

    let port_conf = Value::Array(vec![
        Value::BulkString("replconf".to_string()),
        Value::BulkString("listening-port".to_string()),
        Value::BulkString(srv_read.port.to_string()),
    ]);


    handler.write_value(port_conf).await.unwrap();


    // let resp_conf_port = timeout(time::Duration::from_secs(10), handler.read_value()).await.unwrap().unwrap();
    let resp_conf_port = handler.read_value().await.unwrap();


    match resp_conf_port {
        Some(value) => {
            let (command, _ ) = extract_command(value).unwrap();
            if command.to_lowercase() != "ok" {
                println!("Slave did not receive ok from master");
                return;
            }

            println!("Slave received ok from master")
        }
        None => {
            println!("Slave received null value should got ok");
            return;
        }
    };

    let capa_conf = Value::Array(vec![
        Value::BulkString("replconf".to_string()),
        Value::BulkString("capa".to_string()),
        Value::BulkString("psync2".to_string()),
    ]);

    handler.write_value(capa_conf).await.unwrap();

    // let resp_conf_capa = timeout(time::Duration::from_secs(10), handler.read_value()).await.unwrap().unwrap();
    let resp_conf_capa = handler.read_value().await.unwrap();

    match resp_conf_capa {
        Some(value) => {
            let (command, _ ) = extract_command(value).unwrap();
            if command.to_lowercase() != "ok" {
                println!("Slave did not receive ok from master");
                return;
            }

            println!("Slave received ok from master")
        }
        None => {
            println!("Slave received null value should got ok");
            return;
        }
    };

    let psync_conf = Value::Array(vec![
        Value::BulkString("psync".to_string()),
        Value::BulkString("?".to_string()),
        Value::BulkString("-1".to_string()),
    ]);

    handler.write_value(psync_conf).await.unwrap();
    let resp_conf_psync = timeout(time::Duration::from_secs(10), handler.read_value())
        .await
        .unwrap()
        .unwrap();

    match resp_conf_psync {
        Some(value) => {
            let (command, _ ) = extract_command(value).unwrap();
            println!("Slave received {} from master", command);
        }
        None => {
            println!("Slave received null value should got ok");
            return;
        }
    };


}



// *2\r\n$4\r\necho\r\n$3\r\nhey\r\n
async fn handle_conn(stream: TcpStream, db: &Arc<Database>, srv: &Arc<RwLock<Server>>) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        if value.is_none() {
            println!("Master Received null value");
            break;
        }

        println!("Master Got value: {:?}", value);


        if let Some(value) = value {
            let (command, args) = extract_command(value).unwrap();
            match command.to_lowercase().as_str() {
                "ping" => {
                    let response = Value::SimpleString("PONG".to_string());

                    println!("Master Sending value {:?}", response);
                    handler.write_value(response).await.unwrap();
                },
                "echo" => {
                    let response = args.first().unwrap().clone();
                    println!("Master Sending value {:?}", response);
                    handler.write_value(response).await.unwrap();
                },
                "get" => {
                    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                    let response = match db.get(key.as_str()) {
                        Some(value) => Value::BulkString(value),
                        None => Value::NullBulkString
                    };

                    println!("Master Sending value {:?}", response);
                    handler.write_value(response).await.unwrap();

                }
                "set" => {
                    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                    let value = unpack_bulk_str(args.get(1).unwrap().clone()).unwrap();


                    let response = match args.get(2) {
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

                    };

                    println!("Master Sending value {:?}", response);
                    handler.write_value(response).await.unwrap();


                }
                "info" => {

                    let mut response = Value::NullBulkString;
                    if let Some(command_value) = args.first() {
                        let command_str = unpack_bulk_str(command_value.clone()).unwrap();

                        if command_str.to_lowercase() == "replication" {

                            response = Value::BulkString(format!("role:{}\nmaster_replid:{}\nmaster_repl_offset:{}",
                                                      srv.read().unwrap().role.to_string(),
                                                      srv.read().unwrap().master_replid,
                                                      srv.read().unwrap().master_repl_offset
                            ))
                        }

                    }

                    println!("Master Sending value {:?}", response);
                    handler.write_value(response).await.unwrap();

                }
                "replconf" => {
                    let response = Value::SimpleString("OK".to_string());
                    println!("Master Sending value {:?}", response);
                    handler.write_value(response).await.unwrap();
                }
                "psync" => {
                    let response = Value::SimpleString(format!("+FULLRESYNC {} 0", srv.read().unwrap().master_replid));
                    println!("Master Sending value {:?}", response);
                    handler.write_value(response).await.unwrap();


                    // SEND THE RDB FILE
                    let empty_rdb_hex = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";

                    let empty_rdb_bytes = hex::decode(empty_rdb_hex).unwrap();
                    let empty_rdb_str = String::from_utf8(empty_rdb_bytes.clone()).unwrap();

                    let message = format!("${}\r\n{}", empty_rdb_bytes.len(), empty_rdb_str);
                    handler.write_value(Value::RawString(message)).await.unwrap();

                }
                c => panic!("Unsupported command: {}", c)
            }
        } else {
            println!("Master Received null value");
            break;
        };






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
        Value::SimpleString(s) => {
            Ok((s, vec![]))
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