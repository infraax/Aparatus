# Package quarantine — `rws quarantine`

Aparatus issue #21. Rule: NAP-corpus issue #22 (`docs/ic/UPGRADE-DISCIPLINE.md`). A checker, not an updater:
it records, waits and refuses. **It never writes a lockfile.**

```bash
scripts/quarantine.sh                                   # dev-up.sh if needed, then rws quarantine over the socket
scripts/quarantine.sh --advisory-feed FEED.json         # + advisories → Advice
scripts/quarantine.sh --lockfile path/Cargo.lock        # other lockfile(s), repeatable
scripts/quarantine.sh --wait-days 0 --reason "…"        # override: written as its own receipt
```

## The wait

- **Default: 5 days** (`DEFAULT_WAIT_DAYS`), measured from the registry publish time to the writer's clock.
- A pin younger than the wait is `waiting`. The previous adopted version of that package stays adopted (shown as
  `adopted stays <version>` and stored in the receipt's `adopted` field). Nothing is downgraded or rewritten.
- `--wait-days N` other than 5 needs `--reason`; without it the command exits 1 and records nothing (no silent zero).
  With it, an `event_envelope` (`provenance stated`, payload `{wait_days, default_wait_days: 5, reason}`) is written first.

## What lands on the chain

| Case | Receipt | Rule |
|---|---|---|
| Pin hash matches the registry, age ≥ wait | `event_envelope` `quarantine:<eco>:<name>@<version>:adopted` | measured, evidence **B**, source = registry URL, payload `{ecosystem, name, version, integrity, published_at, age_days, wait_days, status, lockfile}` |
| Hash matches, age < wait (or unknown) | same, `…:waiting`, plus `adopted` = previous adopted version or null | |
| Lockfile hash ≠ registry hash, or ≠ the hash already recorded on the chain | `illegal_transition_rejected` | **Q-01**, exit 2 |
| Registry has no hash for the pin (not found, offline) | `illegal_transition_rejected` | **Q-02**, exit 2 |
| Advisory in the feed matches a pin | `advice` (addressed to `policy`, inputs = the pin receipt) | not applied: advice is not policy |

Each pin is recorded once per state. A rescan records only new pins, new states (waiting → adopted) and refusals.
The hash is checked on **every** scan, including pins already adopted.

## Registries

| Lockfile | Hash in lockfile | Registry fact checked | Source |
|---|---|---|---|
| `Cargo.lock` (registry packages only; path and git deps skipped) | `checksum` (SHA-256 of the `.crate`) | `cksum` and `pubtime` for the version | crates.io sparse index `https://index.crates.io/<prefix>/<name>` |
| `package-lock.json` (lockfileVersion 2/3) | `integrity` | `versions[v].dist.integrity`, `time[v]` | `https://registry.npmjs.org/<name>` |
| `pnpm-lock.yaml` (v6/v9 `packages:` keys) | `resolution.integrity` | same as npm | same |

The `cksum` in the crates.io index is the registry's recorded crate hash; the `.crate` tarball itself is not downloaded.
`--registry-fixture FILE` replaces the registries with a local JSON file (tests, offline).

## Feeds

| Feed | Status |
|---|---|
| Local feed file (`--advisory-feed`, format below) | **built**; `docs/fixtures/quarantine-feed.json` is a fixture, not a real advisory |
| OSV (`api.osv.dev`) | **not built** in this pass |
| RustSec advisory DB | **not built** in this pass |
| GitHub security advisories (only if a token is already in the environment) | **not built** in this pass |

Feed format: `{"feed": "<name>", "advisories": [{"id", "ecosystem": "cargo"|"npm", "package", "bad_versions": [...], "recommended", "url"}]}`.
The Advice summary names the feed, the advisory id, the bad version, the recommended version and the source URL.

## Run on this repo (2026-10-01, daemon path, live crates.io)

```text
$ scripts/quarantine.sh
pin cargo:lazy_static@1.5.1 waiting (age 0d < 5d); adopted stays none
pin cargo:quinn-proto@0.11.19 waiting (age 1d < 5d); adopted stays none
pin cargo:quinn-udp@0.5.16 waiting (age 1d < 5d); adopted stays none
pin cargo:tokio-rustls@0.26.6 waiting (age 3d < 5d); adopted stays none
pin cargo:yoke-derive@0.8.4 waiting (age 1d < 5d); adopted stays none
quarantine: 300 pins, 295 adopted, 5 waiting, 0 refused, 0 already recorded, 0 advice; lockfiles not modified
$ scripts/quarantine.sh      # second run
quarantine: 300 pins, 0 adopted, 0 waiting, 0 refused, 300 already recorded, 0 advice; lockfiles not modified
```

Five of this repo's own pins (pulled in with `ic-agent` in PR #22) were younger than the wait. They are recorded as
`waiting`; `Cargo.lock` is byte-identical before and after (`cmp`). 300 pins took ~6 s (one index request per crate).

Tampered lockfile against the live index (`serde 1.0.229`, already adopted above, checksum `…e0ba` changed to `…e0bb`):

```text
01a0f801-c0b6-7161-a604-5a9fe5998e1f event 51b9be584773d161f3e175c034b47c83c85e016536752814c8c410bdfee29e75
REFUSED Q-01: cargo:serde@1.0.229 hash mismatch: …/Cargo.lock has …e0bb, https://index.crates.io/se/rd/serde has …e0ba
exit 2
```

Fixture advisory against the real pins:

```text
advice fixture:FIXTURE-2026-0001 for cargo:lazy_static@1.5.1
01a0f801-ee76-716e-a3fe-81054f3f27bf advice 65def2aa78bc0f6d32a1b60ac12cfcc635b27925d6cd404b6fe123d270ac264e
```

`Cargo.lock` unchanged; 0 `policy` receipts.

**Bug found and fixed before commit:** the first version skipped already-adopted pins before the hash check, so the
tampered `serde` lockfile above first passed with exit 0. The hash is now checked first on every scan; the CLI test
`quarantine_hash_mismatch_is_refused_on_chain` covers it.

## Tests

- CLI: `quarantine_young_version_waits_and_keeps_previous_pin`, `quarantine_hash_mismatch_is_refused_on_chain`
  (incl. Q-02 and the adopted-pin regression), `quarantine_advisory_writes_advice_and_leaves_lockfile`,
  `quarantine_wait_override_needs_a_reason_and_is_recorded`.
- Daemon: `quarantine_over_rpc_records_and_refuses`.
- Parsers: Cargo.lock, package-lock.json v3 (and v1 refused), pnpm-lock.yaml v9, sparse-index path and versions.
