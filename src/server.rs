// TCP listener and connection handling

use crate::request::HttpRequest;
use crate::response::HttpResponse;
use std::io::{self, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn read_request(stream: &mut TcpStream) -> io::Result<HttpRequest> {
    const MAX_HEADER_SIZE: usize = 16 * 1024;
    const MAX_BODY_SIZE: usize = 1024 * 1024;

    let mut buffer = Vec::new();
    let mut chunk = [0u8; 512];

    // Step 1: Read until the header terminator arrives.
    let header_end = loop {
        let bytes_read = stream.read(&mut chunk)?;

        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Connection closed before complete headers",
            ));
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);

        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }

        if buffer.len() > MAX_HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "HTTP headers too large",
            ));
        }
    };

    // Step 2: Parse only the header section.
    let header_text = std::str::from_utf8(&buffer[..header_end])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in HTTP headers"))?;

    let mut request = HttpRequest::parse(header_text)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    // Step 3: Reject chunked requests for now.
    if request.headers.contains_key("transfer-encoding") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Transfer-Encoding is not supported yet",
        ));
    }

    // Step 4: Read Content-Length.
    let content_length = match request.headers.get("content-length") {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Content-Length"))?,
        None => 0,
    };

    if content_length > MAX_BODY_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Request body too large",
        ));
    }

    // Step 5: Preserve body bytes already received.
    let already_received = &buffer[header_end..];
    let bytes_to_copy = already_received.len().min(content_length);

    request
        .body
        .extend_from_slice(&already_received[..bytes_to_copy]);

    // Step 6: Read the remaining body bytes.
    while request.body.len() < content_length {
        let remaining = content_length - request.body.len();

        let mut body_chunk = vec![0u8; remaining.min(4096)];

        let bytes_read = stream.read(&mut body_chunk)?;

        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Connection closed before complete body",
            ));
        }

        request.body.extend_from_slice(&body_chunk[..bytes_read]);
    }

    Ok(request)
}

fn handle_client(mut stream: TcpStream) {
    match read_request(&mut stream) {
        Ok(request) => {
            println!("Parsed request: {:#?}", request);

            let response = HttpResponse::new(
                200, 
                "application/json".to_string(), 
                b"success\n".to_vec(),
            );

            if let Err(error) = response.send(&mut stream) {
                eprintln!("Failed to send response: {}", error);
            }
        }

        Err(error) => {
            eprintln!("Request error: {}", error);

            let response = HttpResponse::new(
                400, 
                "application/json".to_string(), 
                b"Bad Request\n".to_vec()
            );

            if let Err(error) = response.send(&mut stream) {
                eprintln!("Failed to send error response: {}", error);
            }
        }
    }
}

pub fn start() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to port 8080");

    println!("server listening on 127.0.0.1:8080");

    loop {
        let (stream, address) = listener.accept().expect("failed to accept connection");

        println!("new client connected: {}", address);

        thread::spawn(move || {
            handle_client(stream);
        });
    }
}
