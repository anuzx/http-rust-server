// TCP listener and connection handling

use crate::request::{HttpMethod, HttpRequest};
use crate::response::HttpResponse;
use std::io::{self, Read};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};

fn read_request(stream: &mut TcpStream, buffer: &mut Vec<u8>) -> io::Result<HttpRequest> {
    const MAX_HEADER_SIZE: usize = 16 * 1024;
    const MAX_BODY_SIZE: usize = 1024 * 1024;

    let mut chunk = [0u8; 512];

    // Step 1: Read until the complete headers arrive.
    let header_end = loop {
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }

        if buffer.len() > MAX_HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "HTTP headers too large",
            ));
        }

        let bytes_read = stream.read(&mut chunk)?;

        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Connection closed before complete headers",
            ));
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);
    };

    // Step 2: Parse the header section.
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

    let body_end = header_end + content_length;

    // Step 5: Read until the complete body arrives.
    while buffer.len() < body_end {
        let bytes_read = stream.read(&mut chunk)?;

        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Connection closed before complete body",
            ));
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);
    }

    // Step 6: Copy this request's body.
    request
        .body
        .extend_from_slice(&buffer[header_end..body_end]);

    // Step 7: Remove only the bytes belonging to this request.
    // Any bytes belonging to the next request remain in the buffer.
    buffer.drain(..body_end);

    Ok(request)
}

fn supports_gzip(request: &HttpRequest) -> bool {
    match request.headers.get("accept-encoding") {
        Some(value) => value.split(',').any(|encoding| encoding.trim() == "gzip"),
        None => false,
    }
}

fn handle_client(mut stream: TcpStream) {
    // Reuse this buffer for every request on this connection.
    let mut buffer = Vec::new();

    loop {
        // 1. Read exactly one HTTP request.
        let request = match read_request(&mut stream, &mut buffer) {
            Ok(request) => request,

            Err(error) => {
                if error.kind() == io::ErrorKind::UnexpectedEof {
                    println!("Client disconnected.");
                } else {
                    eprintln!("Request error: {}", error);

                    let response =
                        HttpResponse::new(400, "text/plain".to_string(), b"Bad Request".to_vec());

                    let _ = response.send(&mut stream , false);
                    
                }

                break;
            }
        };

        println!("Parsed request: {:#?}", request);
       
        // 2. Build the response.
        let mut response =
            if matches!(&request.method, HttpMethod::Get) && request.path == "/index.html" {
                match fs::read("public/index.html") {
                    Ok(contents) => HttpResponse::new(200, "text/html".to_string(), contents),

                    Err(error) => {
                        eprintln!("Failed to read HTML file: {}", error);

                        HttpResponse::new(
                            500,
                            "text/plain".to_string(),
                            b"Internal Server Error".to_vec(),
                        )
                    }
                }
            } else {
                HttpResponse::new(200, "text/plain".to_string(), b"Hello from Rust!\n\n".to_vec())
            };

        // 3. Compress if the client supports gzip.
        if supports_gzip(&request) {
            if let Err(error) = response.compress_gzip() {
                eprintln!("Gzip compression failed: {}", error);
            }
        }

        
        
        // 4. Check whether the client requested connection closure.
        let should_close = request
            .headers
            .get("connection")
            .map(|value| {
                value
                    .split(',')
                    .any(|token| token.trim().eq_ignore_ascii_case("close"))
            })
            .unwrap_or(false);
        
        // 5. Send the response with the correct connection header.
        if let Err(error) = response.send(&mut stream, !should_close) {
            eprintln!("Failed to send response: {}", error);
            break;
        }
        
        println!("========== RESPONSE SENT ==========\n");
        
        if should_close {
            println!("Client requested connection closure.");
            break;
        }
        
        println!("Waiting for next request on the same connection...\n");
        println!("========== NEW HTTP REQUEST ==========");

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
