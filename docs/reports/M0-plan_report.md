## Documents Read
- The Dutch Way.md
- Design.md
- Agents.md

## Architecture Constraints Extracted
- Apparatus is local-first, owner-operated, and Rust-first.
- Strict separation of immutable canonical data and disposable derived indices.

## Repository State
- Clean repository, only documentation exists.

## Proposed File Changes
- rust-toolchain.toml, Cargo.toml workspace.
- 7 new foundation crates.

## Proposed Dependencies
- uuid, serde, thiserror, anyhow, sha2, clap.

## Test and Verification Plan
- cargo test --workspace, clippy, run doctor.

## Assumptions / Questions
- I will use Rust 2021 Edition for stability.
