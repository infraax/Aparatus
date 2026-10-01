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

## 5. Long-run config (Part 2): the change and its five-minute proof

**What changed:** Aparatus `scripts/ic-up.sh` (commit `bd24866`).

- Each start renders `.ic-local/replica.active.json5` from the stock `replica.json5`. In the rendered file one line changes:

  ```
  -        exporter: "log",
  +        exporter: { file: "<IC_DIR>/run/metrics-at-shutdown.prom" },
  ```

- The replica then writes **no metrics to `replica.log`**. It dumps them once, on shutdown, to `run/metrics-at-shutdown.prom`. This run produced 913 387 bytes there, with 552 `# HELP` series.
- **Debug flag:** `IC_METRICS_LOG=1 scripts/ic-up.sh` restores the stock `exporter: "log"` for that run only. **Off by default.**
- **Unchanged:**
  - log level `info`;
  - every other line family;
  - `run/backup` and its hourly purge;
  - the checkpoint interval;
  - `state/`.

**The run:** 2026-10-01, 18:55:52 → 19:00:46 UTC, same state as `STATE-GROWTH.md` (18 canisters). Read-only sampling with `stat`, `du`, `ps`, `/api/v2/status` and the ic-agent status read.

| Sample | `replica.log` bytes | lines | `/api/v2/status` | health | certified height | RSS kB |
|---|---|---|---|---|---|---|
| before start | 93 463 834 | 1 093 283 | — | — | — | — |
| **start** (18:55:52) | **93 474 939** | 1 093 324 | 200 | healthy | **5 346** | 89 656 |
| 1 min | 93 477 468 | 1 093 334 | 200 | healthy | 5 472 | 148 396 |
| 2 min | 93 485 886 | 1 093 364 | 200 | healthy | 5 612 | 158 296 |
| 3 min | 93 486 506 | 1 093 366 | 200 | healthy | 5 751 | 158 720 |
| 4 min | 93 487 126 | 1 093 368 | 200 | healthy | 5 891 | 159 984 |
| **5 min** (19:00:46) | **93 495 741** | 1 093 399 | 200 | healthy | **6 026** | 168 472 |

**Log growth, start to 5 min:**

| | Before (exporter `log`, `STATE-GROWTH.md`) | After (exporter `file`) |
|---|---|---|
| Bytes in 5 min | 8 484 342 | **20 802** |
| Rate | ≈ 28.8 kB/s | ≈ 71 B/s |
| Share of the 9 MB measured before | — | **0.24%** |

- With the shutdown lines, the run wrote **78 lines, 21 603 bytes**. None of them are metric lines.
- What still writes, by crate:

  | Crate | Level | Lines |
  |---|---|---|
  | `ic_consensus_dkg/dkg_key_manager` | INFO | 26 |
  | `ic_state_manager` (checkpoint, tip) | INFO | 24 |
  | `ic_query_stats/payload_builder` | WARN | 11 |
  | `ic_state_layout` | INFO | 4 |
  | `ic_http_endpoints_public` | INFO | 4 |
  | `ic_consensus` (`batch_delivery`, `notary`) | INFO/WARN | 4 |
  | start and stop lines | — | 5 |

  These are exactly the families §2 keeps.
- **The certified height kept rising** (5 346 → 6 026), and `/api/v2/status` answered 200 at every sample.

**`run/backup` (not changed by us) moved on its own.**
- The replica's own purge ran soon after start: 10 539 519 bytes at start, 3 406 982 at 1 min. Files older than `retention_time_secs: 3600` went.
- It then climbed again, to 4 039 008 bytes at 5 min.
- This is the replica's retention working. **A five-minute run does not show a plateau**, and none is claimed.

`run/` went from 137 252 159 bytes at start to 130 772 599 at 5 min. The drop is the purge, not the log.
