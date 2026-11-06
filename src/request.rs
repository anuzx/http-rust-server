// HTTP request types

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
}

impl HttpRequest {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let request_line = raw
            .lines()
            .next()
            .ok_or("Empty HTTP request")?;

        let mut parts = request_line.split_whitespace();

        let method = parts
            .next()
            .ok_or("Missing HTTP method")?;

        let path = parts
            .next()
            .ok_or("Missing request path")?;

        let version = parts
            .next()
            .ok_or("Missing HTTP version")?;

        let method = match method {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            _ => return Err("Unsupported HTTP method".to_string()),
        };

        Ok(HttpRequest {
            method,
            path: path.to_string(),
            version: version.to_string(),
        })
    }
}