# Agents — read order and precedence

1. RWS 2.0 policy (`infraax/delta`): `RWS-2.0-MAPPING.md` first (build contract), then `RWS-2.0.md` §2–§6, §8, §11, §12.
2. `README.md` (layout, status).
3. `docs/milestones/M1.md`, `M2.md`, `M3.md`: how the runtime runs.
4. `docs/handoff/M4-HANDOFF.md`: the next task, with its brief.
5. `docs/notes/M*-DEFERRED.md`: what is still open.
6. The code: `crates/*/src/`, `bins/*/src/`.

Precedence: **RWS 2.0 + MAPPING overrule `docs/background/` (Design.md, Dutch Way).** Background is not required reading.

Rules for agents working here:

- Do not invent receipt kinds beyond the eleven in RWS-2.0 X-02, or event kinds beyond F-02 ∪ X-12.
- Anything needed but out of scope goes to the current milestone's `docs/notes/M*-DEFERRED.md`, not into code.
- Agents are clients: they write through `apparatusd` (or the CLI) and never keep the chain as their own memory.
- `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` and `cargo build --workspace --locked` stay green.
