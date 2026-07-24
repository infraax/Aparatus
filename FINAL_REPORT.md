## Implementation Summary
Created the minimal M0 foundational Rust workspace for Apparatus per the provided guidelines, setting up stable Rust build infrastructure and exactly the specified packages with strictly controlled dependencies.

## Exact Files Created/Changed
- `.editorconfig`, `.gitignore`, `.vscode/extensions.json`, `.vscode/settings.json`, `Cargo.toml`, `justfile`, `README.md`
- `crates/apparatus-types/Cargo.toml`, `crates/apparatus-types/src/lib.rs`
- `crates/apparatus-time/Cargo.toml`, `crates/apparatus-time/src/lib.rs`
- `crates/apparatus-crypto/Cargo.toml`, `crates/apparatus-crypto/src/lib.rs`
- `crates/apparatus-schema/Cargo.toml`, `crates/apparatus-schema/src/lib.rs`
- `crates/apparatus-store/Cargo.toml`, `crates/apparatus-store/src/lib.rs`
- `crates/apparatus-artifacts/Cargo.toml`, `crates/apparatus-artifacts/src/lib.rs`
- `crates/apparatus-ledger/Cargo.toml`, `crates/apparatus-ledger/src/lib.rs`
- `bins/apparatus-cli/Cargo.toml`, `bins/apparatus-cli/src/main.rs`

## Crates and Dependency Direction
- `apparatus-types`: Depend on `serde`, `uuid`, `thiserror`.
- `apparatus-time`: Depend on `time`.
- `apparatus-crypto`: Depend on `sha2`, `thiserror`, `serde` (optional via feature).
- `apparatus-schema`: Depend on `apparatus-types`, `thiserror`.
- `apparatus-store`: Depend on `apparatus-types`, `thiserror`.
- `apparatus-artifacts`: Depend on `apparatus-types`, `apparatus-crypto`, `thiserror`.
- `apparatus-ledger`: Depend on `apparatus-types`, `apparatus-crypto`, `serde`, `serde_json`, `thiserror`.
- `apparatus-cli`: Depend on `clap`, `anyhow`.

Dependency direction flows one-way from foundation up to the CLI.

## Tests Added
- **apparatus-types**: Validated UUIDv7 round-trip serialization and distinct `ObjectHeader` identities.
- **apparatus-time**: Demonstrated deterministic test clock advances vs normal system clock.
- **apparatus-crypto**: Passed SHA-256 known answer validation and lowercase hex extraction.
- **apparatus-schema**: Validated empty timestamp and provenance invariants on restricted classification structs.
- **apparatus-artifacts**: Tested derivation and strict string pattern extraction of CAS relative paths.
- **apparatus-ledger**: Verified serialization hashes across identical operation payloads remain deterministic.

## Verification Commands
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace --locked`
- `cargo run -p apparatus-cli -- --version`
- `cargo run -p apparatus-cli -- doctor`
- Output correctly reflected zero lint warnings, all 13 passing unit tests, and correctly executable diagnostic commands.

## Unavailable Commands
- `cargo deny check`
- `cargo audit`
- (These cargo extensions are not installed in this environment but are safely left for CI).

## Deferred Work
- Real CAS filesystem interactions, `rusqlite` SQL migrations, chain verification logic, and Cedar policy engines.

## Risks/Assumptions
- Serde's default JSON payload canonicalization requires stable schema structures. A stronger guarantee via BTreeMap is recommended for future expansions where dictionaries arrive from unknown endpoints.
