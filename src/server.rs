// TCP listener and connection handling

use crate::request::{self, HttpRequest};
use std::io::{self, Read};
use std::net::{TcpListener, TcpStream};

fn read_request_headers(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 512];

    const MAX_HEADER_SIZE: usize = 16 * 1024;

    loop {
        let bytes_read = stream.read(&mut chunk)?;

        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Client closed connection before sending complete headers",
            ));
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);

        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            return Ok(buffer);
        }

        if buffer.len() > MAX_HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "HTTP headers are too large",
            ));
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let raw_bytes = match read_request_headers(&mut stream) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("Failed to read request: {}", error);
            return;
        }
    };

    let raw_request = String::from_utf8_lossy(&raw_bytes);

    match HttpRequest::parse(&raw_request) {
        Ok(request) => {
            println!("Parsed request: {:#?}", request);
            
            if let Some(content_type) = request.headers.get("content-type") {
                println!("Content-Type: {}", content_type);
            }

            if let Some(content_length) = request.headers.get("content-length") {
                println!("Content-Length: {}", content_length);
            }
        }

        Err(error) => {
            println!("Failed to parse request: {}", error);
        }
    }
}

pub fn start() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to port 8080");

    println!("server listening on 127.0.0.1:8080");

    loop {
        let (stream, address) = listener.accept().expect("failed to accept connection");

        println!("new client connected: {}", address);

        handle_client(stream);
    }
}
