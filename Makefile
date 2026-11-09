.PHONY: server test test-one

# Run Rust HTTP server
server:
	cargo run

# Run all Go tests with colours
test:
	cd tests && richgo test -v

# Run a specific Go test
# Usage: make test-one TEST=TestGzipCompression
test-one:
	cd tests && richgo test -v -run '^$(TEST)$$'
