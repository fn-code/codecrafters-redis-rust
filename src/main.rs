// Uncomment this block to pass the first stage

mod resp;

use tokio::net::{TcpListener,TcpStream};
use crate::resp::Value;
use anyhow::Result;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("0.0.0.0:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {

                    handle_conn(stream).await;
                });

            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

// *2\r\n$4\r\necho\r\n$3\r\nhey\r\n
async fn handle_conn(stream: TcpStream) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();
        println!("Got value: {:?}", value);

        let response = if let Some(value) = value {
            let (command, args) = extract_command(value).unwrap();
            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
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