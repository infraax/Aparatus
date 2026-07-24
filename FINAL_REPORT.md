## Implementation Summary
Hardened the M0 foundational components per `Design.md` and the prompt:
- Expanded `ObjectHeader` to strictly type all required fields (object type, schema version, status, detailed provenance, content digest, supersession, causation, correlation).
- Hardened domain ID types, replacing `Default` with parsing strictly from string or UUID, and ensuring only UUIDv7 is accepted for `ObjectId`.
- Refactored `apparatus-time` to return `UnixMs` wrapper avoiding `expect` and handling checked addition/conversions.
- Hardened `Sha256Digest` and `validate_and_extract_hash_from_cas_path` string parsing logic.
- Transformed `Receipt` hashing away from opaque `serde_json::Value` by defining `ReceiptHashPreimageV1`.

## Exact Files Changed
- `crates/apparatus-types/src/lib.rs`, `crates/apparatus-types/Cargo.toml`
- `crates/apparatus-time/src/lib.rs`, `crates/apparatus-time/Cargo.toml`
- `crates/apparatus-crypto/src/lib.rs`, `crates/apparatus-crypto/Cargo.toml`
- `crates/apparatus-schema/src/lib.rs`, `crates/apparatus-schema/Cargo.toml`
- `crates/apparatus-artifacts/src/lib.rs`
- `crates/apparatus-ledger/src/lib.rs`, `crates/apparatus-ledger/Cargo.toml`
- `.gitignore`, `justfile`, `.vscode/extensions.json`

## Public API Breaking Changes
- `ObjectHeader` structure completely revised to include missing requirements.
- `Receipt` structure updated (`previous_hash` type and removed `operation_payload` in favor of `operation_payload_digest`).
- Default trait implementations removed from domain ID wrapper types like `ProjectId`, `ArtifactId`, etc. Instead, initialization goes through explicit parsing.
- Clock `.now_ms()` signature changed to `Result<UnixMs, TimeError>`.

## Design.md requirements now satisfied
- Strict domain boundaries preserved.
- Local-first canonical object header metadata completeness.
- Deterministic cryptographic hashing formats set in stone before persistence work.

## Dependencies added or explicitly declined
- `apparatus-types` added `apparatus-time` to use `UnixMs`.
- Added `vadimcn.vscode-lldb` to the workspace recommendations for developer ergonomics.
- Declined: Any persistence layer, tokio, SQLite, web dependencies, or MCP tools per boundaries.

## Tests added and what each proves
- `apparatus-types`: Proves `ContentDigest` rejects bad lengths/characters. Proves `ObjectId` rejects v4 UUIDs, allowing only v7.
- `apparatus-time`: Proves `DeterministicClock` throws an error and does not wrap when advancing past `u64::MAX`.
- `apparatus-crypto`: Proves `Sha256Digest` rejects uppercase and non-hex inputs.
- `apparatus-schema`: Proves invalid timestamp ordering and incomplete provenance invariants return errors.
- `apparatus-artifacts`: Proves extraction rejects trailing slashes, path traversals (`..`), absolute paths (`/`), Windows slashes (`\`), incorrect parts, and mismatched shards.
- `apparatus-ledger`: Proves changing optional fields in the ledger receipt modifies its deterministic hash output.

## Exact verification commands and outcomes
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace --locked`
- All run successfully. No unused import warnings remaining.

## Deferred work
- `rusqlite` persistence, task definitions, complete CI matrix testing.

## Open design questions requiring owner review
- `ObjectStatus` is defined, but policy lifecycle transitions (e.g., active -> superseded) are not yet validated by the schema engine (since it lacks historical state). Review if status transitions belong in future policy layers.
