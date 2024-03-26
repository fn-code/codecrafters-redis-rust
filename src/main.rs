// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("0.0.0.0:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                handle_client(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: std::net::TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        let read_count = stream
            .read(&mut buffer)
            .expect("failed to read data from client");
        println!("received data: {:?}", String::from_utf8_lossy(&buffer[..read_count]));

        if read_count == 0 {
           break;
        }

        stream
            .write_all(b"+PONG\r\n")
            .expect("failed to write data to client");

        stream.flush().expect("failed to flush data to client");
    }

}
