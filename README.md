# Apparatus

Apparatus is a local-first, owner-operated system preserving canonical, append-only history.

This repository contains the M0 (Foundational) implementation for the system, laying out the immutable object, cryptographic hashing, and local ledger architecture schemas.

## Development

Prerequisites:
- `rustup` configured for the stable Rust channel
- `just` task runner (optional but recommended)

### Workflows

To format code:
```bash
cargo fmt --all
# or
just fmt-fix
```

To run lint checks:
```bash
cargo clippy --workspace --all-targets -- -D warnings
# or
just lint
```

To run all unit and integration tests:
```bash
cargo test --workspace
# or
just test
```

To build the workspace:
```bash
cargo build --workspace --locked
# or
just build
```

To run the CLI diagnostics:
```bash
cargo run -p apparatus-cli -- doctor
```

## Architecture Notes

* **Crates**
  * `apparatus-types`: Core identity and data structure definitions (ObjectIds, ObjectHeader).
  * `apparatus-time`: Pluggable clock trait for deterministic testing.
  * `apparatus-crypto`: Narrow cryptographic operations for artifact identifiers (SHA-256).
  * `apparatus-schema`: Validation traits and internal invariant checks for canonical data structures.
  * `apparatus-store`: Persistence traits separating schema definitions from backend logic (to be backed by SQLite/CAS).
  * `apparatus-artifacts`: CAS path generation and identifier routines.
  * `apparatus-ledger`: Append-only chaining definitions and deterministic hashing.
  * `apparatus-cli`: Core system interface.

* **Design Invariants**
  * Data is append-only and artifacts are immutable.
  * SQLite is the intended backing datastore.
  * Dependency trees are strictly controlled and phase-gated.
