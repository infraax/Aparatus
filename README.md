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

Runtime: `M1.md` (kernel, CLI `apparatus`), `M2.md` (agreements, lock), `M3.md` (daemon `apparatusd`).

To run the CLI diagnostics:
```bash
cargo run -p apparatus-cli -- doctor
```

## Architecture Notes

* **Crates**
  * `apparatus-types`: Core identity and data structure definitions (ObjectIds, ObjectHeader).
  * `apparatus-time`: Pluggable clock trait for deterministic testing.
  * `apparatus-crypto`: Narrow cryptographic operations for artifact identifiers (SHA-256).
  * `apparatus-schema`: Validation traits; RWS 2.0 payload rules and the chain `State`.
  * `apparatus-store`: Persistence traits and errors. M1 implementations: `FsArtifactStore` (artifacts), `FileLedger` (ledger).
  * `apparatus-artifacts`: CAS path generation and the write-once filesystem store.
  * `apparatus-ledger`: Receipts, canonical-JSON hashing, append-only JSONL chain with HEAD.
  * `apparatus-kernel`: the single-writer RWS kernel, request/response API and socket client.
  * `apparatus-cli`: `doctor` and the `rws` commands (daemon-first client).
  * `apparatusd`: always-on JSONL RPC daemon around one writer actor.

* **Design Invariants**
  * Data is append-only and artifacts are immutable.
  * SQLite is the intended backing datastore.
  * Dependency trees are strictly controlled and phase-gated.
