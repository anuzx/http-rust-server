// TCP listener and connection handling

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use crate::request::HttpRequest;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    let bytes_read = stream
        .read(&mut buffer)
        .expect("Failed to read from stream");

    let raw_request = String::from_utf8_lossy(&buffer[..bytes_read]);

    match HttpRequest::parse(&raw_request) {
        Ok(request) => {
            println!("Parsed request: {:?}", request);
        }
        Err(error) => {
            println!("Failed to parse request: {}", error);
        }
    }
}

pub fn start() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to binf to port 8080");

    println!("server listening on 127.0.0.1:8080");

    loop {
        let (stream , address) = listener.accept().expect("failed to accept connection");

        println!("new client connected: {}" , address);

        handle_client(stream);
    }
}
