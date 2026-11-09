package main

import (
	"bufio"
	"bytes"
	"compress/gzip"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"strings"
	"sync"
	"testing"
	"time"
)

func serverAddress() string {
	if addr := os.Getenv("SERVER_ADDR"); addr != "" {
		return addr
	}
	return "127.0.0.1:8080"
}

func baseURL() string {
	return "http://" + serverAddress()
}

func newHTTPClient() *http.Client {
	return &http.Client{
		Timeout: 5 * time.Second,
	}
}

// Reads one complete HTTP response from a raw TCP connection.
func readRawResponse(
	reader *bufio.Reader,
	method string,
) (*http.Response, []byte, error) {
	req := &http.Request{Method: method}

	resp, err := http.ReadResponse(reader, req)
	if err != nil {
		return nil, nil, err
	}

	body, err := io.ReadAll(resp.Body)
	resp.Body.Close()

	if err != nil {
		return resp, nil, err
	}

	return resp, body, nil
}

// 1. Test a basic GET request.
func TestGetRequest(t *testing.T) {
	client := newHTTPClient()

	resp, err := client.Get(baseURL() + "/")
	if err != nil {
		t.Fatalf("GET request failed: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("failed to read response body: %v", err)
	}

	if len(body) == 0 {
		t.Fatal("expected a non-empty response body")
	}

	t.Logf("GET response: %s", string(body))
}

// 2. Test a POST request with a body.
func TestPostRequest(t *testing.T) {
	client := newHTTPClient()

	requestBody := `{"name":"Anuj","role":"backend engineer"}`

	resp, err := client.Post(
		baseURL()+"/users",
		"application/json",
		strings.NewReader(requestBody),
	)
	if err != nil {
		t.Fatalf("POST request failed: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("failed to read response body: %v", err)
	}

	t.Logf("POST response: %s", string(body))
}

// 3. Test serving public/index.html.
func TestGetIndexHTML(t *testing.T) {
	client := newHTTPClient()

	resp, err := client.Get(baseURL() + "/index.html")
	if err != nil {
		t.Fatalf("GET /index.html failed: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", resp.StatusCode)
	}

	if !strings.Contains(resp.Header.Get("Content-Type"), "text/html") {
		t.Fatalf(
			"expected HTML Content-Type, got %q",
			resp.Header.Get("Content-Type"),
		)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("failed to read HTML body: %v", err)
	}

	if !bytes.Contains(body, []byte("<html")) &&
		!bytes.Contains(body, []byte("<!doctype html")) {
		t.Fatal("response does not appear to contain HTML")
	}

	t.Logf("Received %d bytes of HTML", len(body))
}

// 4. Test five concurrent requests.
func TestFiveConcurrentRequests(t *testing.T) {
	const numberOfRequests = 5

	client := &http.Client{
		Timeout: 5 * time.Second,
		Transport: &http.Transport{
			DisableKeepAlives: true,
		},
	}

	var wg sync.WaitGroup
	start := make(chan struct{})
	errors := make(chan error, numberOfRequests)

	for i := 0; i < numberOfRequests; i++ {
		wg.Add(1)

		go func(id int) {
			defer wg.Done()

			<-start

			resp, err := client.Get(baseURL() + "/")
			if err != nil {
				errors <- fmt.Errorf("request %d failed: %w", id, err)
				return
			}
			defer resp.Body.Close()

			if resp.StatusCode != http.StatusOK {
				errors <- fmt.Errorf(
					"request %d: expected status 200, got %d",
					id,
					resp.StatusCode,
				)
				return
			}

			_, err = io.Copy(io.Discard, resp.Body)
			if err != nil {
				errors <- fmt.Errorf(
					"request %d: failed to read body: %w",
					id,
					err,
				)
			}
		}(i + 1)
	}

	close(start)
	wg.Wait()
	close(errors)

	for err := range errors {
		t.Error(err)
	}

	t.Log("All five concurrent requests completed")
}

// 5. Test gzip compression of index.html.
func TestGzipCompression(t *testing.T) {
	// Fetch the normal, uncompressed HTML.
	normalResp, err := newHTTPClient().Get(baseURL() + "/index.html")
	if err != nil {
		t.Fatalf("normal GET failed: %v", err)
	}

	normalBody, err := io.ReadAll(normalResp.Body)
	normalResp.Body.Close()

	if err != nil {
		t.Fatalf("failed to read normal response: %v", err)
	}

	if normalResp.StatusCode != http.StatusOK {
		t.Fatalf("normal GET returned status %d", normalResp.StatusCode)
	}

	// Disable automatic gzip handling.
	// We want to inspect the compressed bytes ourselves.
	client := &http.Client{
		Timeout: 5 * time.Second,
		Transport: &http.Transport{
			DisableCompression: true,
		},
	}

	req, err := http.NewRequest(
		http.MethodGet,
		baseURL()+"/index.html",
		nil,
	)
	if err != nil {
		t.Fatal(err)
	}

	req.Header.Set("Accept-Encoding", "gzip")

	resp, err := client.Do(req)
	if err != nil {
		t.Fatalf("gzip GET failed: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", resp.StatusCode)
	}

	if resp.Header.Get("Content-Encoding") != "gzip" {
		t.Fatalf(
			"expected Content-Encoding: gzip, got %q",
			resp.Header.Get("Content-Encoding"),
		)
	}

	compressedBody, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("failed to read compressed body: %v", err)
	}

	gzipReader, err := gzip.NewReader(bytes.NewReader(compressedBody))
	if err != nil {
		t.Fatalf("failed to create gzip reader: %v", err)
	}
	defer gzipReader.Close()

	decompressedBody, err := io.ReadAll(gzipReader)
	if err != nil {
		t.Fatalf("failed to decompress response: %v", err)
	}

	if !bytes.Equal(normalBody, decompressedBody) {
		t.Fatal("decompressed HTML does not match the normal HTML response")
	}

	if len(compressedBody) >= len(normalBody) {
		t.Log("Note: compressed body is not smaller than the original body")
	}

	t.Logf(
		"Original: %d bytes | Compressed: %d bytes",
		len(normalBody),
		len(compressedBody),
	)
}

// 6. Test multiple requests over one persistent TCP connection.
func TestPersistentConnection(t *testing.T) {
	conn, err := net.DialTimeout("tcp", serverAddress(), 3*time.Second)
	if err != nil {
		t.Fatalf("TCP connection failed: %v", err)
	}
	defer conn.Close()

	conn.SetDeadline(time.Now().Add(5 * time.Second))

	reader := bufio.NewReader(conn)

	// First request.
	_, err = fmt.Fprintf(
		conn,
		"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n",
	)
	if err != nil {
		t.Fatalf("failed to send first request: %v", err)
	}

	resp1, _, err := readRawResponse(reader, http.MethodGet)
	if err != nil {
		t.Fatalf("failed to read first response: %v", err)
	}

	if resp1.StatusCode != http.StatusOK {
		t.Fatalf("first response: expected 200, got %d", resp1.StatusCode)
	}

	if !strings.EqualFold(resp1.Header.Get("Connection"), "keep-alive") {
		t.Fatalf(
			"expected first response to keep connection alive, got %q",
			resp1.Header.Get("Connection"),
		)
	}

	// Second request on the SAME TCP connection.
	_, err = fmt.Fprintf(
		conn,
		"GET /index.html HTTP/1.1\r\nHost: localhost\r\n\r\n",
	)
	if err != nil {
		t.Fatalf("failed to send second request: %v", err)
	}

	resp2, body2, err := readRawResponse(reader, http.MethodGet)
	if err != nil {
		t.Fatalf("failed to read second response: %v", err)
	}

	if resp2.StatusCode != http.StatusOK {
		t.Fatalf("second response: expected 200, got %d", resp2.StatusCode)
	}

	if len(body2) == 0 {
		t.Fatal("second response body is empty")
	}

	t.Log("Two requests succeeded over one TCP connection")
}

// 7. Test that Connection: close closes the TCP connection.
func TestConnectionClose(t *testing.T) {
	conn, err := net.DialTimeout("tcp", serverAddress(), 3*time.Second)
	if err != nil {
		t.Fatalf("TCP connection failed: %v", err)
	}
	defer conn.Close()

	reader := bufio.NewReader(conn)

	_, err = fmt.Fprintf(
		conn,
		"GET / HTTP/1.1\r\n"+
			"Host: localhost\r\n"+
			"Connection: close\r\n"+
			"\r\n",
	)
	if err != nil {
		t.Fatalf("failed to send request: %v", err)
	}

	resp, _, err := readRawResponse(reader, http.MethodGet)
	if err != nil {
		t.Fatalf("failed to read response: %v", err)
	}

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", resp.StatusCode)
	}

	// Go represents the Connection: close behaviour
	// using the Response.Close field.
	if !resp.Close {
		t.Fatal("expected response to indicate connection closure")
	}

	// Verify that the server actually closed the TCP connection.
	conn.SetReadDeadline(time.Now().Add(3 * time.Second))

	var oneByte [1]byte
	_, err = conn.Read(oneByte[:])

	if err != io.EOF {
		t.Fatalf("expected EOF after connection close, got %v", err)
	}

	t.Log("Server closed the TCP connection successfully")
}
