# Hygiene (Aparatus #26)

State on 2026-10-02.

## Receipt kinds

There are **twelve** receipt kinds:
- the eleven of RWS-2.0 X-02;
- plus `event_envelope` (NAP-corpus #12).

Event kinds are F-02 ∪ X-12. `envelope` (without `event_`) is only the RWS **means** object (a budget envelope), not a receipt kind. `Agents.md` says the same; nothing may add a kind.

## Read order for a homelab session

1. `Agents.md`: the rules for agents in this repo.
2. `docs/NAP-CORPUS-COMPAT.md`: what is built, what comes next, and in which order.
3. `docs/homelab/CANISTERS.md` and `docs/homelab/NODE-UP.md`: the canisters on the local replica, and the machines.
4. The open issue you are working on.

The `docs/homelab/ANKER.md` named in the issue was never written. `NODE-UP.md` (role `anker`) and `TAILNET-READS.md` cover it.

## The replica is not started by dfx

The replica is started by `scripts/ic-up.sh`: the stock `release-2026-09-25_03-28-base` binaries, checked against `SHA256SUMS`. Canisters are installed by NAP-corpus `tools/ic-install` (ic-agent).
- No doc outside `legacy/` says that dfx starts the replica.
- The two remaining mentions of dfx both say it is *not* used (`scripts/README.md`, `docs/homelab/CANISTERS.md`).

## Branches on origin

Nothing is deleted. A branch is only marked here.

| Branch | Head | In `main`? | Owner / note |
|---|---|---|---|
| `main` | `a614c84` | — | default |
| `claude/homelab` | moves | open PR (Batch 1) | the working branch; merges through a PR |
| `claude/envelope-stage-0-h2g36k` | `afccfca` | merged | Envelope Stage 0 |
| `claude/envelope-stage-1` | `c83a382` | merged | Envelope Stage 1 |
| `claude/envelope-stage-1-validate` | `7c41d68` | merged | Stage 1 validation |
| `claude/ic-status` | `9a26a5d` | merged | IC status client |
| `claude/quarantine` | `3e94b82` | merged | quarantine |
| `m1-rws-runtime`, `m2-keep-agreement`, `m3-daemon`, `m4-handoff` | — | merged | milestones M1–M4 |
| `apparatus-m0-scaffold-12896545540571922923` | `2ce9706` | **not merged** (1 commit) | "M0 Foundation Hardening", superseded by M1–M3; kept for history |
| `claude/repo-pull-handoff-lalxvj` | `bc185c5` | **not merged** (1 commit) | the M4 drop-pipeline draft (Aparatus #5/#6); still open work, not merged |
