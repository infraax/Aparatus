# M4 handoff — conduits (done: see `docs/milestones/M4.md`)

**State at handoff (2026-09-30):** `main` contains M1–M3 (PRs #2, #3 merged) plus this cleanup. 54 tests green, clippy/fmt/`--locked` clean.
M4 was **not** implemented: the previous session ran out of budget. Start here.

## Start

```bash
git checkout main && git pull
git checkout -b m4-conduits
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings   # must be green first
```

Read: `docs/milestones/M3.md`, `docs/notes/M3-DEFERRED.md`, `Agents.md`, RWS-2.0 §8 (canon), §11 (telemetry), §4 (keep, short).
Do not merge the PR unless the owner asks in the same message.

## Brief (owner's, condensed — do not rename code after Dutch terms)

M3 = one daemon, one writer, tickets. That stays. M4 = ordered streams so corpus and bytes do not rot, measurement,
a backup that is never source of truth, and empty conduits for work five years out. Keep it working first; smarter later.

| RWS | M4 |
|---|---|
| Keep ≠ project | Corpus ingest is **keep**: a boring pipe, no gate theatre |
| Work window | Ingest queue with one active batch; never many writers on one file |
| Capacity > envelope | Queue full → `busy` + ticket, no second ledger |
| Condition report | Periodic `check` + signals on the chain |
| Replica, no rights | Backup dir is a replica (`replica: true`); restore is an ingest-like procedure |
| Empty conduit | Hook trait + events; no S3/MCP/cluster now |
| Failure first-class | Pipeline failure = ticket + event; daemon stays up |

### 1. Ingest drop-pipeline (required)

`<project>/.apparatus/in/{new,work,ok,refuse,quarantine}/`
- Only the daemon moves `new → work`; without a daemon, `rws ingest-dir` does the same under LOCK.
- One file at a time in `work/`. Crash in `work/`: re-offer on start, or quarantine + ticket. Never a half-admitted CAS object.
- Same digest already admitted → refuse or idempotent no-op, never a second artefact id.
- Optional sidecar `file.md.rws.json` {purpose, tag, classification}. Missing tag → `C` + ticket "untagged ingest".
- `refuse/` gets a sidecar `.json` with the rule. Agents only drop into `in/new/`; they never write CAS.
- RPC `ingest_enqueue` {path | bytes+name} feeds the same pipe.
- Test: drop 3 files incl. one duplicate digest and one empty file → 2 ok, 1 refuse, `check` green, no orphan counted as admitted.

### 2. Signals and failsafes

- Daemon tick (default 60 s, `--signal-every`): failing check → event + ticket, **no exit**.
  Dampen: at most one fail-ticket per check cause until it is green again.
- `.apparatus/SIGNAL`: last status, time, queue depth, head hash (watchdog-readable without RPC). `rws signals` reads it or asks the daemon.
- Keep: stale-socket handling, writer panic → reload + ticket. `busy` → document "check `head`"; optional single CLI retry.
- Disk full / CAS write failure → ticket, no process exit unless start is impossible.
- No Prometheus. `on_signal` hook is the conduit for later scraping.

### 3. Backup / restore (replica)

`apparatus rws backup --to DIR` · `apparatus rws restore --from DIR --project EMPTYDIR`
- Consistent copy under lock or via the daemon: HEAD + ledger.jsonl + cas/ + project.json, plus a manifest (time, head id, head hash, file count; `replica: true`).
- Restore only into an empty `.apparatus/`; never over a live HEAD. No encryption/offsite/rclone.
- Test: backup, mutate source, restore to temp, `check` head == backup head, source unchanged.

### 4. Hooks (empty conduits)

Module or crate `apparatus-hooks`: trait `Hook: Send + Sync` with default no-op `after_receipt(&Receipt)`,
`after_refuse(rule, subject)`, `on_signal(&Signal)`, `after_backup(&Path)`. Ship `NullHook` (+ optional `LogHook` to stderr).
Zero or one hook; no plugin ABI, no dlopen. Future MCP-notify / Tailscale-wake / object storage implement it — do not build them.

### Not in M4

MCP server, TCP 0.0.0.0, tokens, SQLite, multi-project daemon, scheduler crate (`target_missed`), Redis, HA, auto-merge,
Design.md / Dutch Way edits, new receipt kinds (tickets stay `report_back`).

### Deliverables

Branch `m4-conduits`. `docs/milestones/M4.md` ≤ 100 lines. `docs/notes/M4-DEFERRED.md` (MCP hook, offsite backup,
Prometheus, SQLite, night ingest windows). `Agents.md` lists M4.md. Existing 54 tests stay green; local mode without daemon keeps working.
Commit: `Add ingest drop-pipeline, health signals, replica backup, hook trait.`

Report: pipe folders, what a crash in `work/` does, backup consistency, whether the tick spams tickets (and the damping).

## Pointers into the code

- Writer actor + socket: `bins/apparatusd/src/daemon.rs` (`writer_loop`, `serve`, `submit`) — add the ingest worker and tick here.
- Request/Response + `execute`: `crates/apparatus-kernel/src/api.rs` (add `IngestEnqueue`, `Signals`, `Backup` ops).
- Kernel (lock, submit, CAS side effect): `crates/apparatus-kernel/src/kernel.rs`.
- CAS: `crates/apparatus-artifacts` (`FsArtifactStore::put` is write-once). Ledger lock: `crates/apparatus-ledger` (`FileLedger::lock`).
