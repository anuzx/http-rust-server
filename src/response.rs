// HTTP response types

use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::{self, Write};
use std::net::TcpStream;

pub struct HttpResponse {
    pub status_code: u16,
    pub content_type: String,
    pub content_encoding: Option<String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status_code: u16, content_type: String, body: Vec<u8>) -> Self {
        Self {
            status_code,
            content_type,
            content_encoding: None,
            body,
        }
    }

    pub fn compress_gzip(&mut self) -> io::Result<()> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

        encoder.write_all(&self.body)?;

        let compressed_body = encoder.finish()?;

        self.body = compressed_body;
        self.content_encoding = Some("gzip".to_string());

        Ok(())
    }

    pub fn send(&self, stream: &mut TcpStream) -> io::Result<()> {
        let status_text = match self.status_code {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };

        let mut headers = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n",
            self.status_code,
            status_text,
            self.content_type,
            self.body.len(),
        );

        // Add Content-Encoding before ending the headers.
        if let Some(encoding) = &self.content_encoding {
            headers.push_str(&format!("Content-Encoding: {}\r\n", encoding));
        }

        // End the HTTP header section.
        headers.push_str("\r\n");

        // Send headers first, then body.
        stream.write_all(headers.as_bytes())?;
        stream.write_all(&self.body)?;

        Ok(())
    }
}
