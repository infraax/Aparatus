# Agents — read order and precedence

1. RWS 2.0 policy (`infraax/delta`, branch `claude/rws-2-0`): `RWS-2.0-MAPPING.md` first (build contract), then `RWS-2.0.md` §2, §3, §4, §5, §6, §8, §12.
2. `M1.md` in this repo: how the runtime runs.
3. `M2.md`: keep agreements, solo mandate_holder, correction/review CLI, writer lock.
4. `M3.md`: `apparatusd`, JSONL RPC, one writer actor, tickets.
5. `NOTES/M3-DEFERRED.md` and `NOTES/M2-DEFERRED.md`: what is still open.
6. The code: `crates/*/src/lib.rs`, `crates/apparatus-kernel/src/`, `bins/*/src/`.

Precedence: **RWS 2.0 + MAPPING overrule `Design.md` and the Dutch Way documents.**
`Design.md` and the Dutch Way files are background only; they are no longer required reading.

Rules for agents working here:

- Do not invent receipt kinds beyond the eleven in RWS-2.0 X-02, or event kinds beyond F-02 ∪ X-12.
- Anything needed but out of scope goes to `NOTES/M3-DEFERRED.md`, not into code.
- Agents are clients: they write through `apparatusd` (or the CLI) and never keep the chain as their own memory.
- `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` must stay green.
