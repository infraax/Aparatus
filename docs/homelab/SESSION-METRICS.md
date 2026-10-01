# Session metrics (2026-10-01, cloud sandbox, local N=1 replica)

**Sandbox:** 4 vCPU, 16 GiB RAM. At session start, 466 MiB of RAM was in use and the disk was 55% full (21 G of 252 G).

**Replica:** stock `replica` from `release-2026-09-25_03-28-base`, state in `.ic-local/`. It was started only for install tests and stopped after each.

## Replica runs

| Run | Start (UTC) | Stop (UTC) | Minutes up | RSS at start | RSS at stop | Certified height at stop | `.ic-local` at stop |
|---|---|---|---|---|---|---|---|
| 1: Parts 3–6 (lifeline, install order, mirror/register, public groups) | 17:47:41 | 17:58:03 | 10.4 | 85 220 kB (83 MiB) | 188 912 kB (184 MiB) | 4 424 | 455 M |
| 2: Part 7 (mcp-gate, bridge run) | 18:02:18 | 18:03:12 | 0.9 | 172 420 kB (168 MiB) | **177 032 kB (173 MiB)** | **4 537** | 462 M |

**Totals**
- The replica was up for **11.3 minutes**. The highest certified height reached was **4 537**.
  - The replica resumes from its checkpoints in `.ic-local/run`. Height carries across restarts and sessions.
- apparatusd used 11 352 kB of RSS at the run-2 stop.

**Disk**

| | Session start | Session end |
|---|---|---|
| `.ic-local` | 412 M | 462 M |
| of which `run/` (replica state) | — | 167 M |
| Root filesystem | 55% (21 G) | 59% (23 G) |

The extra filesystem use is mostly Cargo build output and `node_modules` for the mops packages.

## Killed for slowness

**Nothing.** The sandbox did not slow down, so no process was killed and no compile was interrupted.

- Every stop was orderly: `scripts/ic-down.sh` (SIGTERM, wait) and `scripts/dev-down.sh`.
- The replica stayed stopped during the long Rust builds (`dev-check.sh`, about 20 s each when cached).

## At the end of the session

```text
$ pgrep -a 'replica|apparatusd|orchestrator'
no replica, apparatusd or orchestrator process
```
