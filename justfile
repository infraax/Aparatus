# Format code
fmt:
    cargo fmt --all -- --check

# Format and fix code
fmt-fix:
    cargo fmt --all

# Run clippy lints
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run tests
test:
    cargo test --workspace

# Build the workspace
build:
    cargo build --workspace --locked

# Check the workspace (format, lint, test)
check: fmt lint test

# Run cargo deny
deny:
    cargo deny check

# Run cargo audit
audit:
    cargo audit
