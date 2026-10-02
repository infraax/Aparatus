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
  When the registry gives no publish time, the wait counts from the local first sight (below).
- A pin younger than the wait is `waiting`. The previous adopted version of that package stays adopted (shown as
  `adopted stays <version>` and stored in the receipt's `adopted` field). Nothing is downgraded or rewritten.
- `--wait-days N` other than 5 needs `--reason`; without it the command exits 1 and records nothing (no silent zero).
  With it, an `event_envelope` (`provenance stated`, payload `{wait_days, default_wait_days: 5, reason}`) is written first.

## What lands on the chain

| Case | Receipt | Rule |
|---|---|---|
| Pin hash matches the registry, age ≥ wait | `event_envelope` `quarantine:<eco>:<name>@<version>:adopted` | measured, evidence **B**, source = registry URL, payload `{ecosystem, name, version, integrity, published_at, age_days, wait_days, status, lockfile}` |
| Hash matches, age < wait | same, `…:waiting`, plus `adopted` = previous adopted version or null | |
| First sight of a version not yet adopted | `event_envelope` `…:seen`, payload `{registry_integrity, registry, published_at, first_seen_at}` | measured, evidence **B** (the witness, B1.8) |
| Registry hash ≠ the hash it had at first sight | `illegal_transition_rejected` | **Q-01**, exit 2 |
| Lockfile hash ≠ registry hash, or ≠ the hash already recorded on the chain | `illegal_transition_rejected` | **Q-01**, exit 2 |
| Registry has no hash for the pin (not found, offline) | `illegal_transition_rejected` | **Q-02**, exit 2 |
| Advisory in the feed matches a pin | `advice` (addressed to `policy`, inputs = the pin receipt) | not applied: advice is not policy |

Each pin is recorded once per state. A rescan records only new pins, new states (waiting → adopted) and refusals.
The hash is checked on **every** scan, including pins already adopted.

## Registry-hash witness (B1.8, 2026-10-02)

The 5-day wait trusts the registry's own publish time. The witness adds evidence that does not:
- The first time a version is seen (and not yet adopted), the writer records the registry's hash and its own clock (`first_seen_at`).
- A registry that later serves a different hash for the same version is refused (Q-01), even if the lockfile follows it.
- A version without a publish time is no longer stuck in `waiting`: it waits 5 days from first sight.
- `adopted` and `waiting` receipts now carry `first_seen_at`. Adopted pins get no witness (their hash is already on the chain).

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
| OSV (`api.osv.dev`), live: `--osv` | **built** (2026-10-01). One `POST /v1/querybatch` for all pins (chunks of 500), then `GET /v1/vulns/<id>` per distinct id. Withdrawn advisories are skipped. `recommended` = the lowest `fixed` version above the pin |
| RustSec advisory DB | **built, through OSV**. OSV serves the RustSec database (`RUSTSEC-*` ids); those Advice name the feed `rustsec (via osv)` and link `rustsec.org/advisories/<id>`. When OSV returns a GHSA record that a RUSTSEC record lists as an alias for the same pin, the GHSA one is dropped |
| GitHub security advisories | **covered through OSV** (GHSA ids), no token needed. A direct GitHub API feed is not built, and no new secret is required |

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

## Live feed run (2026-10-01)

`--osv` adds live advisories to the pins being scanned; `--advisory-feed` still works alongside it. Every match is **Advice only**: nothing is applied, and no lockfile is written. The checksums below are the same before and after.

On this repo's own `Cargo.lock`, OSV returns two RustSec advisories:
- `paste` 1.0.15: `RUSTSEC-2024-0436`
- `serde_cbor` 0.11.2: `RUSTSEC-2021-0127`

Both are "unmaintained" advisories. Both crates come in through ic-agent. They stay pinned, and replacing them is a later Policy decision.

`docs/fixtures/osv-live/Cargo.lock` is a one-pin lockfile: `smallvec` 0.6.9 with its real crates.io checksum. The run gives four RustSec Advice records. The GHSA aliases are dropped.

```text
$ sha256sum Cargo.lock docs/fixtures/osv-live/Cargo.lock
e3a9bf9a430a056f1a61c8dc3cb1fa3ebda2ddca640509bba23c37f7de76b902  Cargo.lock
08cc45655066f763fb1684e02ed90116b62a970133dd0e410c502100f683b6bb  docs/fixtures/osv-live/Cargo.lock
$ target/debug/apparatus rws quarantine --osv   # this repo, live crates.io + live OSV
osv: 2 advisories for 300 pins
advice rustsec (via osv):RUSTSEC-2024-0436 for cargo:paste@1.0.15
advice rustsec (via osv):RUSTSEC-2021-0127 for cargo:serde_cbor@0.11.2
quarantine: 300 pins, 0 adopted, 0 waiting, 0 refused, 300 already recorded, 2 advice; lockfiles not modified
exit 0
$ target/debug/apparatus rws quarantine --osv --lockfile docs/fixtures/osv-live/Cargo.lock   # one known-bad pin (smallvec 0.6.9, real crates.io checksum)
osv: 4 advisories for 1 pins
01a0f8a8-2e58-76b6-99ca-943827d0b845 event_envelope 0930a5d763530b3f4c07f9aa9f8823350eba33a28c0a677f1cb605dcf176aff5
advice rustsec (via osv):RUSTSEC-2018-0018 for cargo:smallvec@0.6.9
01a0f8a8-2e81-7414-9592-9fab5be3d0c6 advice 97a8034985ef79aa2047af84d3e0acd5b6788b49f44488fbfc1bc06335a56de0
advice rustsec (via osv):RUSTSEC-2019-0009 for cargo:smallvec@0.6.9
01a0f8a8-2e84-7543-8483-f570c76dcf61 advice ce355bfa3c8eea588a0d76d8ec74a93926e2e99d241e36ba6f1f1dd28e8e6ce4
advice rustsec (via osv):RUSTSEC-2019-0012 for cargo:smallvec@0.6.9
01a0f8a8-2e86-7165-8104-930aed511d12 advice ae4d0c7b3e60ac7dab2ae9d3ec66bb72f80f8ef526f5b07bafd3d6e54d15f0a8
advice rustsec (via osv):RUSTSEC-2021-0003 for cargo:smallvec@0.6.9
01a0f8a8-2e87-7492-8870-e5c5d2e04356 advice ff95396aea36298f1429f9e041e3a3aa9916c99039d49d5290e78eee7397b525
quarantine: 1 pins, 1 adopted, 0 waiting, 0 refused, 0 already recorded, 4 advice; lockfiles not modified
exit 0
$ sha256sum -c
Cargo.lock: OK
docs/fixtures/osv-live/Cargo.lock: OK
```

## Tests

- `osv_batch_body_and_hits_round_trip`, `osv_record_names_feed_id_and_lowest_fix_and_dedupes_the_ghsa_alias`: the OSV request/response handling, offline, no network.

- CLI: `quarantine_young_version_waits_and_keeps_previous_pin`, `quarantine_hash_mismatch_is_refused_on_chain`
  (incl. Q-02 and the adopted-pin regression), `quarantine_advisory_writes_advice_and_leaves_lockfile`,
  `quarantine_wait_override_needs_a_reason_and_is_recorded`.
- Daemon: `quarantine_over_rpc_records_and_refuses`, `quarantine_witness_records_first_sight_and_refuses_a_changed_registry_hash`.
- Parsers: Cargo.lock, package-lock.json v3 (and v1 refused), pnpm-lock.yaml v9, sparse-index path and versions.
