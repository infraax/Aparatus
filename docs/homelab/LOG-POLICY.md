# Replica log policy (NAP-corpus #48)

This policy covers the local stock replica: `release-2026-09-25_03-28-base`, started by Aparatus `scripts/ic-up.sh`. The same file is in NAP-corpus `docs/ic/LOG-POLICY.md` and Aparatus `docs/homelab/LOG-POLICY.md`.

## 1. What the log contains (measured, before any change)

`run/replica.log` after the growth run in `STATE-GROWTH.md`: **1 093 283 lines, 93 463 834 bytes**. It is one file, appended to across every start since 2026-10-01 15:28 UTC.

How lines were counted: by format and by crate (`s:<subnet>/n:<node>/<crate>/<module>`) with a one-pass script. There are no `ERRO` or `CRIT` lines in the file.

| Family | Level | Lines | Bytes | Share of bytes | Example |
|---|---|---|---|---|---|
| **Metrics** (`exporter: "log"`, Prometheus text) | — | 1 091 287 | 92 955 905 | **99.46%** | `replica_http_call_v3_certificate_status_total{status="rejected"} 1` |
| Consensus (`ic_consensus_*`, `ic_artifact_pool`) | WARN | 1 011 | 232 298 | 0.25% | `ic_consensus_dkg/utils Reusing current transcript for tag LowThreshold` |
| Consensus | INFO | 186 | 53 871 | 0.06% | `ic_artifact_pool/lmdb_pool PersistentIDkgPool…` |
| Execution (`ic_execution_environment`) | INFO | 217 | 83 722 | 0.09% | `canister_manager` install and upgrade lines |
| State manager (`ic_state_manager`, `ic_state_layout`) | INFO | 265 | 68 839 | 0.07% | `Created unverified checkpoint @5000`, `Recovering checkpoint @4500 as tip` |
| State manager | WARN | 6 | 1 380 | 0.00% | `No state available with certification.` (at start) |
| Startup / infra (`ic_replica`, `ic_crypto`, `ic_http_*`, `ic_quic`, …) | INFO | 164 | 44 577 | 0.05% | `Found subnet … with nodes […]`, `Replica Started` |
| Query stats (`ic_query_stats`) | WARN/INFO | 67 | 20 842 | 0.02% | `Current stats are uninitialized` |
| **Errors** (`ERRO`, `CRIT`) | — | **0** | 0 | 0% | — |
| **Canister rejects / traps** as text lines | — | **0** | 0 | 0% | A reject is a client response, not a log line. In the log it appears only as metric counters, e.g. `…certificate_status_total{status="rejected"}` |
| Unparsed (`, Application: MetricsRuntime`, the tail of a metrics header) | — | 80 | 2 400 | 0.00% | — |

**74 metric dumps, ≈ 1.26 MB each.** The exporter writes the full registry about every 30 s while the replica runs. That is the 28.8 kB/s measured in `STATE-GROWTH.md`.

## 2. What a long-running replica needs, and what is only for a debug run

| Family | Long run | Why |
|---|---|---|
| Errors (`ERRO`/`CRIT`) | **keep** | none so far, but they are the only record of a fault the replica saw itself |
| Consensus INFO/WARN | **keep** | DKG, CUP and pool events, a few kB a minute |
| State manager | **keep** | checkpoint creation and recovery heights; this is how `STATE-GROWTH.md` saw @4500 → @5000 |
| Execution, startup and infra, query stats | **keep** | small; install and upgrade lines name the canister |
| **Metrics** | **debug only** | See the reasons below |
| Canister rejects | nothing to keep in the log | Each reject is returned to the caller, and our canisters store their refusals as Events (book, daily root, mirror, mcp-gate) |

Why metrics are debug only:
- Health and height come from `/api/v2/status` (`replica_health_status`, `certified_height`) and need no metrics.
- The per-call reject counters are aggregates that duplicate what the caller already received.
- The HTTP exporter cannot run here at all. It binds `[::]` and panics on an IPv4-only host (`POC-NOTES.md`, *"Could not start TCP listener at addr = [::]:9090"*).

What the long run loses without metrics in the log: the *finalization* height that `LOCAL-RUN.md` reads from the metric dump. The certified height is still in `/api/v2/status`.

## 3. The claim "a failure can be reconstructed from certified state or the state root plus a dev flag"

**For the replica itself: [NF].** `replica --help` lists only these options:

`--print-sample-config`, `--config-file`, `--config-literal`, `--catch-up-package`, `--guestos-version`, `--replica-version`, `--force-subnet`

None of them replays or reconstructs a failure. The sample config's `logger` block offers `level` (critical … trace), `format` (`text_full` | `json`) and `block_on_overflow`, and nothing else.

**What does exist is a separate stock tool, `ic-replay`.** I fetched it from the same release and verified it against `SHA256SUMS` (`ic-replay.gz: OK`). It was **not run** against this state. From `ic-replay --help`:

> `--replay-until-height <REPLAY_UNTIL_HEIGHT>` The replay will stop at this height, deliver potential extra batches and finally deliver one final extra batch to make a checkpoint.
>
> `restore-from-backup` Restore from the backup
>
> `--replica-version …` Required if no consensus pool is available, otherwise the version is taken from its finalized tip

What that means:
- Replay re-executes from a **checkpoint plus finalized blocks**, taken from `run/pool` or from the artifact backup `run/backup`.
- It does **not** work from the certified state or the state root alone.
- It regenerates **state, not log text**. A lost log line cannot be recovered by replay. A lost state can, if the pool or the backup spool still holds the blocks.
- So `run/backup` is the input to recovery and is **left alone**: the replica's own hourly purge stays, and nothing of ours moves it.
- Error and consensus lines are **not dropped**, because nothing reconstructs them.

## 4. Decision (before the run)

- **Drop only the metrics family from `replica.log` in long runs.**
- **Keep every other family**, at the stock `level: "info"`.
- **Do not change** `run/backup`, the checkpoint interval, or `state/`.
- The change and its five-minute proof are below (Part 2).
