# Rust HTTP Server

A lightweight HTTP/1.1 server built from scratch using **Rust TCP primitives**.

This project explores how HTTP works under the hood, including TCP connections, HTTP request parsing, response construction, concurrency, persistent connections, and gzip compression.

The server handles basic GET and POST requests, serves static HTML files, and supports multiple requests over persistent TCP connections.

## What is an HTTP Server?

An HTTP server is a software application that receives HTTP requests from clients and sends HTTP responses.

For example, when you visit a website:

1. Your browser acts as an HTTP client.
2. It establishes a TCP connection with the server.
3. It sends an HTTP request.
4. The server processes the request.
5. The server sends an HTTP response.
6. The browser processes the response and displays the content.

A basic HTTP request looks like this:

```http
GET /index.html HTTP/1.1
Host: localhost:8080
Accept: text/html
```

A basic HTTP response looks like this:

```http
HTTP/1.1 200 OK
Content-Type: text/html
Content-Length: 286
Connection: keep-alive

<!doctype html>
...
```

This project implements these HTTP fundamentals directly over TCP instead of relying on an HTTP framework.

## Features

### 1. TCP-based HTTP Server

- Built using Rust's `std::net::TcpListener` and `TcpStream`.
- Listens on `127.0.0.1:8080`.
- Parses HTTP request lines, headers, and request bodies.
- Constructs HTTP responses manually.
- Supports basic GET and POST requests.

### 2. Concurrent Connections

The server uses Rust threads to handle multiple TCP connections concurrently.

Each incoming connection gets its own thread using `std::thread::spawn`.

This allows the server to handle multiple clients without waiting for one client to finish before accepting another.

### 3. Static File Serving

The server can serve static HTML files from the `public/` directory.

Currently supported:

```http
GET /index.html HTTP/1.1
```

The server reads the file and returns its contents with the appropriate content type.

### 4. Gzip Compression

The server supports gzip compression using the Rust `flate2` crate.

The compression flow:

1. The client sends `Accept-Encoding: gzip`.
2. The server checks whether gzip is supported.
3. The server compresses the response body.
4. The server sends `Content-Encoding: gzip`.
5. The server calculates `Content-Length` using the compressed body size.

Example:

```http
Accept-Encoding: gzip
```

Server response:

```http
Content-Encoding: gzip
```

This reduces response payload size when compression is beneficial.

### 5. Persistent Connections

The server supports HTTP/1.1 persistent connections.

Instead of closing the TCP connection after every response, the server can process multiple HTTP requests over the same connection.

For example:

```text
TCP Connection
    |
    |---- GET /
    |<--- HTTP 200
    |
    |---- GET /index.html
    |<--- HTTP 200
    |
    |---- GET /
    |<--- HTTP 200
    |
    Connection remains open
```

This avoids establishing a new TCP connection for every request.

### 6. Graceful Connection Closure

The server supports the `Connection: close` header.

When a client sends:

```http
GET / HTTP/1.1
Host: localhost
Connection: close
```

The server:

1. Detects the connection closure request.
2. Sends the response with `Connection: close`.
3. Stops processing further requests on that connection.
4. Closes the TCP connection when the handler finishes.

This ensures that the connection is closed after the response has been sent.


## Getting Started

### Prerequisites

Install the following:

- Rust and Cargo
- Go
- Make
- richgo (for coloured test output)

Install richgo:

```bash
go install github.com/kyoh86/richgo@latest
```

Make sure Go's binary directory is in your PATH:

```bash
export PATH="$PATH:$(go env GOPATH)/bin"
```

### 1. Clone the repository

```bash
git clone <your-repository-url>
cd http-rust-server
```

### 2. Run the server

```bash
make server
```

The server starts at:

```text
http://127.0.0.1:8080
```

### 3. Run all test cases

Open another terminal in the project root:

```bash
make test
```

### 4. Run a specific test

Use `make test-one` followed by the test name.

```bash
make test-one TEST=TestGetRequest
```

Examples:

```bash
make test-one TEST=TestPostRequest
make test-one TEST=TestGetIndexHTML
make test-one TEST=TestFiveConcurrentRequests
make test-one TEST=TestGzipCompression
make test-one TEST=TestPersistentConnection
make test-one TEST=TestConnectionClose
```

## Makefile Commands

| Command | Description |
|---|---|
| `make server` | Start the Rust HTTP server |
| `make test` | Run all Go integration tests |
| `make test-one TEST=TestGetRequest` | Run a specific test |

## Integration Tests

The project includes Go integration tests that communicate with the running Rust server.

| Test | Description |
|---|---|
| `TestGetRequest` | Tests a basic GET request |
| `TestPostRequest` | Tests a POST request with a JSON body |
| `TestGetIndexHTML` | Tests static HTML file serving |
| `TestFiveConcurrentRequests` | Tests five concurrent TCP connections |
| `TestGzipCompression` | Tests gzip compression and decompression |
| `TestPersistentConnection` | Tests multiple requests over one TCP connection |
| `TestConnectionClose` | Tests graceful TCP connection closure |

The tests use Go's `net/http` package for standard HTTP requests and `net.Conn` for low-level TCP connection tests.

<img width="859" height="623" alt="image" src="https://github.com/user-attachments/assets/e88e54c3-05dc-49a6-8f0c-42a623346614" />

