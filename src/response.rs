// HTTP response types

use std::io::{self, Write};
use std::net::TcpStream;

pub struct HttpResponse {
    pub status_code: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(
        status_code: u16,
        content_type: String,
        body: Vec<u8>,
    ) -> Self {
        Self {
            status_code,
            content_type,
            body,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) -> io::Result<()> {
        let status_text = match self.status_code {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };

        let headers = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n",
            self.status_code,
            status_text,
            self.content_type,
            self.body.len(),
        );

        stream.write_all(headers.as_bytes())?;
        stream.write_all(&self.body)?;

        Ok(())
    }
}
