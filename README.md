# Apparatus

A local-first runtime for **RWS 2.0**, run by its owner. The policy text is in `infraax/delta` (`RWS-2.0.md` and `RWS-2.0-MAPPING.md`).

- **One project = one ledger.** Each project has one append-only, hash-chained receipt ledger and a content-addressed artefact store.
- **One writer.** `apparatusd` is the only process that writes. Its socket is mode 0600. The CLI talks to it, or takes the writer lock itself when no daemon runs.
- **Twelve receipt kinds.** The eleven of RWS-2.0 X-02 plus `event_envelope`. `envelope` alone is only the RWS means object (see `docs/homelab/HYGIENE.md`).
- **Advice is not policy.** Agents, feeds and automation may only write Advice. Only an adopted Policy binds.

The research behind it is in `infraax/NAP-corpus`. The canisters it talks to, and the building blocks they share, are in NAP-corpus `docs/ic/` and `infraax/lcrc`.

## What works

| Area | What | Doc |
|---|---|---|
| Ledger and kernel | Typed receipts (RWS 2.0 vocabulary), canonical hashing, JSONL chain and CAS, keep agreements (K-01), corrections and reviews | `docs/milestones/M1.md`–`M3.md` |
| Daemon | `apparatusd`: JSONL RPC on a Unix socket, one writer actor, tickets | `docs/milestones/M3.md` |
| Event envelopes | Provenance and lifecycle (draft → signed → sealed), CID v1 (BLAKE3), Ed25519 signing, one CID per payload | `docs/milestones/ENVELOPE-STAGE-0.md`, `-1.md` |
| IC status | `rws ic status`: reads the local replica with `ic-agent` 0.49.2 and records the certified height as a measured envelope. Replica down → refusal IC-01 | `docs/milestones/IC-STATUS.md` |
| Package quarantine | `rws quarantine`: every lockfile pin hash-checked against its registry; a 5-day wait; a first-seen witness per version; feed advisories become Advice. Never writes a lockfile | `docs/milestones/QUARANTINE.md` |
| MCP bridge | `apparatus mcp serve`: stdio MCP server. The mcp-gate canister checks every tool call before it runs | `docs/milestones/M6.md` |
| Homelab scripts | Local replica (stock binaries, verified), daemon, quarantine, log cap, and an install script for any Linux node | `scripts/README.md`, `docs/homelab/` |

The gate is `scripts/dev-check.sh`: `cargo test --workspace`, `clippy -D warnings`, `fmt --check` and `build --locked`. On 2026-10-02 it ran 103 tests, all passing.

## Quick start

```bash
cargo build --workspace --locked          # target/debug/apparatus, target/debug/apparatusd
apparatus rws init --solo                 # in a project directory
apparatus rws policy agreement --asset '*'
apparatus rws ingest ./README.md --purpose corpus --tag C
apparatus rws event-envelope --data '{"n":1}' --source me --module demo --provenance stated
apparatus rws check                       # verify the chain, CAS and buffer; list refusals
```

With the homelab pieces:

```bash
scripts/ic-up.sh                          # local IC replica on 127.0.0.1:8080
scripts/dev-up.sh                         # apparatusd for this repo
target/debug/apparatus rws ic status      # record the replica's certified height
scripts/quarantine.sh                     # check this repo's own lockfiles
scripts/dev-down.sh && scripts/ic-down.sh
```

## A new machine

`scripts/node-up.sh` installs any Linux node: the Oracle VPS, the anker ThinkPad, a Raspberry Pi or a laptop.
- It measures the machine, picks a tier from NAP-corpus `docs/ic/NODES.md`, and prints the plan.
- It changes nothing without `--apply`.
- See `docs/homelab/NODE-UP.md`.

```bash
scripts/node-up.sh --nodes-md ../NAP-corpus/docs/ic/NODES.md --role pi --bootstrap --units
```

## Layout

```text
crates/
  apparatus-types       ids, headers, RWS 2.0 vocabulary and payloads, quarantine records
  apparatus-time        clocks
  apparatus-crypto      SHA-256, canonical JSON, BLAKE3/CID v1, Ed25519 (ML-DSA-65 stub)
  apparatus-schema      payload validation and chain state (all RWS rules)
  apparatus-store       store traits and errors
  apparatus-artifacts   write-once filesystem CAS
  apparatus-ledger      receipts, JSONL chain, writer lock
  apparatus-kernel      single-writer kernel, Request/Response API, socket client
  apparatus-ic          read-only client for the local replica (ic-agent)
  apparatus-quarantine  lockfile parsers, registry and OSV lookups (client half of rws quarantine)
bins/
  apparatus-cli         `apparatus`: CLI (daemon first) and the MCP bridge
  apparatusd            the always-on writer
scripts/                replica, daemon, quarantine, log cap, node install (see scripts/README.md)
docs/
  milestones/           how each part works and how to run it
  homelab/              replica, canisters, nodes, Tailscale reads, session records
  notes/ handoff/ reports/ background/
legacy/2026-10-01-pre-batch0/   frozen copy of the sources before Batch 0, with a sha256 manifest
.claude/skills/         DFINITY ICP Skills for canister work (pinned in ICSKILLS.lock)
```

Project data lives in `<project>/.apparatus/` (`ledger.jsonl`, `HEAD`, `cas/`, `keys/`, `LOCK`, `apparatusd.sock`). The replica's state is in `.ic-local/`. Both are git-ignored and never committed.
