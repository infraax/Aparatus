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

# Check the workspace
check:
    cargo check --workspace

# Run cargo deny
deny:
    cargo deny check

# Run cargo audit
audit:
    cargo audit

# Run apparatusd under a restart loop: restarts on crash (exit != 0), stops on owner stop (exit 0)
daemon project=".":
    #!/usr/bin/env bash
    until cargo run -q -p apparatusd -- --project "{{project}}"; do
        echo "apparatusd exited with $?; restarting in 2s" >&2
        sleep 2
    done
