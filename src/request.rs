// HTTP request types

use std::collections::HashMap;

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let mut lines = raw.split("\r\n");
        let request_line = lines.next().ok_or("Empty HTTP request")?;

        let mut parts = request_line.split_whitespace();

        let method = parts.next().ok_or("Missing HTTP method")?;

        let path = parts.next().ok_or("Missing request path")?;

        let version = parts.next().ok_or("Missing HTTP version")?;

        let method = match method {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            _ => return Err("Unsupported HTTP method".to_string()),
        };

        //parse headers :
        let mut headers = HashMap::new();

        for line in lines {
            //empty line means headers have ended
            if line.is_empty() {
                break;
            }

            let (name, value) = line.split_once(":").ok_or("invalid http header")?;

            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_string();

            if headers.contains_key(&name) {
                return Err(format!("Duplicate HTTP header: {}", name));
            }

            headers.insert(name, value);
        }

        Ok(HttpRequest {
            method,
            path: path.to_string(),
            version: version.to_string(),
            headers,
        })
    }
}
