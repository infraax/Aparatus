# Apparatus

Local-first, owner-operated runtime for **RWS 2.0** (policy: `infraax/delta`, `RWS-2.0.md` + `RWS-2.0-MAPPING.md`).
One project = one append-only, hash-chained receipt ledger + a content-addressed artefact store, written by exactly one writer.

## Status

| Milestone | What | Doc |
|---|---|---|
| M0 | Workspace scaffold, types, hashing | `docs/reports/` |
| M1 | Kernel: typed payloads (11 kinds), canonical hash, file ledger + CAS, `apparatus rws` CLI | `docs/milestones/M1.md` |
| M2 | Keep agreements (K-01), solo mandate_holder, correction/review, writer lock | `docs/milestones/M2.md` |
| M3 | `apparatusd`: JSONL RPC on a Unix socket, one writer actor, tickets | `docs/milestones/M3.md` |
| M4 | Ingest drop-pipeline, health signals + SIGNAL file, replica backup/restore, `Hook` trait | `docs/milestones/M4.md` |

Open items per milestone: `docs/notes/M*-DEFERRED.md`.

## Quick start

```bash
cargo build --workspace --locked          # binaries: target/debug/apparatus, target/debug/apparatusd
apparatus rws init --solo                 # in a project directory
apparatus rws policy agreement --asset '*'
apparatus rws ingest ./README.md --purpose corpus --tag C
apparatus rws queue keep --asset demo
apparatus rws check
apparatusd --project .                    # optional: always-on daemon; the CLI then talks to it
cp notes.md .apparatus/in/new/            # drop-pipeline (daemon), or `apparatus rws ingest-dir`
apparatus rws signals                     # health; watchdogs read .apparatus/SIGNAL
apparatus rws backup --to ../backup-1     # replica; `rws restore --from` into an empty project
```

## Develop

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                    # 65 tests at M4
just daemon .                             # restart loop around apparatusd
```

## Layout

```text
crates/
  apparatus-types       ids, headers, RWS 2.0 vocabulary and payloads
  apparatus-time        clocks
  apparatus-crypto      SHA-256 digests
  apparatus-schema      payload validation + chain State (all RWS rules)
  apparatus-store       store traits and errors
  apparatus-artifacts   write-once filesystem CAS
  apparatus-ledger      receipts, canonical JSON, JSONL chain, writer lock
  apparatus-hooks       `Hook` trait (empty conduits), `Signal`, NullHook/LogHook
  apparatus-kernel      single-writer kernel, Request/Response API, socket client, ingest pipe, signals, backup
bins/
  apparatus-cli         `apparatus` — daemon-first CLI (local mode under LOCK otherwise)
  apparatusd            always-on JSONL RPC daemon
docs/
  milestones/           M1–M4 how-to-run
  notes/                deferred items per milestone
  handoff/              brief for the next session
  reports/              M0 reports
  background/           Design.md and Dutch Way (background only; RWS 2.0 overrules)
```

Project data lives in `<project>/.apparatus/` (`project.json`, `ledger.jsonl`, `HEAD`, `cas/`, `LOCK`, `apparatusd.sock`, `SIGNAL`, `in/`).
