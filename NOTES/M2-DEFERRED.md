# M2 — deferred

Open after M2. Carried over from `NOTES/M1-DEFERRED.md` unless marked *new*. Each line names the RWS 2.0 rule.

## Validation not yet enforced

- K-01 (*new*): an agreement is active until superseded; agreements carry no period, so expiry is not modelled. Scope matching is exact name or `*` only (no prefixes or patterns).
- K-02 / K-06: no programme artefact; `signalled → firm` is allowed without a programme refresh.
- K-06: `deferred` is terminal (the manual's table has no way back). Needs a rule decision before adding one.
- B-04 / M-05: no-go/stop does not auto-release means; `means_not_released` is not emitted (needs the scheduler).
- B-07: external acts (`deployed`, `contract_signed`) do not yet require `gate_ref`.
- B-09: `envelope_overrun` is recordable but not detected.
- B-12: programme admission rules are not checked.
- C-06 (*new*): the mapping's "evaluator output only by evaluator" is not enforced; there is no `evaluator` role in the v1 enum.
- M-07 / M-08 / M-09: timing shifts, carry-over at period end and overprogramming limits beyond creation are not modelled.
- A-06: `respond_by` is stored; `advice_unanswered` is not emitted (scheduler).
- A-07: authoritative-source registry and `report_back` resolution are not modelled.
- A-08: replica flag is enforced for gate evidence and authority only; no generation-time check.
- R-09: roles are project-scoped only; asset/envelope/incident scopes and `from`/`until` periods are not modelled.
- R-10: coordinator role exists; `escalated` / `de_escalated` pairing is not checked.
- F-03 / V-01: `target_missed`, `deadline_missed`, `gate_abandoned` need the scheduler.
- V-05: agreement supersession mid-period is not restricted to allowed triggers.

- Replay versioning (*new*): history is re-validated under the current rules, so rule tightening (M2: K-01, K-09) breaks replay of older chains. Needs rule versions keyed on the payload `rws` field before real data exists.

## CLI surface not built

- No `policy framework` subcommand (priority groups stay out of the kernel).
- Multi-signer gates in one CLI call: still hand-built payloads.

## Runtime not built

- `apparatus-maintenance` scheduler; SQLite; HTTP/MCP/tokens; Cedar; capability grants; leases.

## Known limits of the file store

- CAS write and ledger append are two steps: a crash between them leaves an orphan blob (harmless; `check` verifies only admitted artefacts).
- The writer lock is advisory: it protects against other `apparatus` processes, not against editing `.apparatus/` by hand (that is caught by `rws check`).
- `show … | head` panics on a closed pipe (standard Rust `println!` behaviour).
