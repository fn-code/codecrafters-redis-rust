// Uncomment this block to pass the first stage

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

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
            Ok((mut stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {

                    handle_client(stream).await;
                });

            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: tokio::net::TcpStream) {
    let mut buffer = [0; 1024];
    loop {


        let read_count = stream.read(&mut buffer).await.unwrap();

        println!("received data: {:?}", String::from_utf8_lossy(&buffer[..read_count]));

        if read_count == 0 {
           break;
        }

        stream.write_all(b"+PONG\r\n").await.unwrap();

        // stream.flush().await.unwrap();
    }

}
