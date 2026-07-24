## Documents Read
- The Dutch Way.md
- Design.md
- Agents.md
- crates/apparatus-types/src/lib.rs
- crates/apparatus-crypto/src/lib.rs
- crates/apparatus-schema/src/lib.rs
- crates/apparatus-time/src/lib.rs
- crates/apparatus-artifacts/src/lib.rs
- crates/apparatus-ledger/src/lib.rs
- bins/apparatus-cli/src/main.rs
- justfile, .gitignore, README.md, .vscode/

## Architecture Constraints Extracted
- Apparatus is local-first, owner-operated, and Rust-first.
- Strict separation of canonical data.
- ObjectHeader must be fully populated with ID, type, schema version, created_at, recorded_at, occurred_at (opt), principal ID, project ID (opt), classification, provenance, source references, status, supersession (opt), correlation (opt), causation (opt).
- No  cyclic coupling.
- Remove unwrap/expect from prod paths, reject bad artifacts/hex strictly, etc.

## Current-Code Gaps
-  is missing required fields (type, schema version, status, recorded_at, etc.)
-  doesn't implement / properly.
-  has .
-  from_hex parsing isn't perfectly strict.
-  doesn't handle absolute paths/windows paths/etc.
-  hash uses  payload directly instead of a strict preimage.

## Proposed API Changes
- Add  and  wrapper to .
- Add missing fields/enums to .
- Add  and change  hashing.
- Refactor  validation.
- Remove  on ID types (replace with explicit generation if needed, but the prompt advises caution. I will implement Display/FromStr instead and remove Default from newtypes).

## Proposed File Changes
- Edit  across all 6 foundation crates.
- Edit  (remove ).
- Edit  and .

## Proposed Dependencies
- None new, sticking to existing (uuid, serde, thiserror, time, sha2).

## Test and Verification Plan
- cargo test --workspace, cargo clippy, cargo run doctor

## Assumptions / Questions
- I will assume standard uuid v7 representation for  parsing.
