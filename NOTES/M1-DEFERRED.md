# M1 — deferred

> Status after M2: items marked **[M2 done]** are closed; everything still open moved to `NOTES/M2-DEFERRED.md`.

Things M1 needed or touched but deliberately did not build. Each line names the RWS 2.0 rule.

## Validation not yet enforced

- **[M2 done]** K-01: `queue_keep` does not yet require an active agreement covering the asset (the demo queues keep work before any agreement exists).
- K-02 / K-06: no programme artefact; `signalled → firm` is allowed without a programme refresh.
- K-06: `deferred` is terminal (the manual's table has no way back). Needs a rule decision before adding one.
- **[M2 done]** K-09: `deferred` does not require a paired `displacement` event.
- B-04 / M-05: no-go/stop does not auto-release means; `means_not_released` is not emitted (needs the scheduler).
- B-07: external acts (`deployed`, `contract_signed`) do not yet require `gate_ref`.
- B-09: `envelope_overrun` is recordable but not detected.
- B-12: programme admission rules are not checked (field exists on `queue_build`).
- M-07 / M-08 / M-09: timing shifts, carry-over at period end and overprogramming limits beyond creation are not modelled.
- A-06: `respond_by` is stored; `advice_unanswered` is not emitted (scheduler).
- A-07: authoritative-source registry and `report_back` resolution are not modelled.
- A-08: replica flag is enforced for gate evidence and authority only; no generation-time check.
- R-09: roles are project-scoped only; `scope.asset_ref` / `envelope_ref` / `incident_ref` and `from`/`until` periods are not modelled.
- R-10: coordinator role exists; `escalated` / `de_escalated` pairing is not checked.
- F-03 / V-01: `target_missed`, `deadline_missed`, `gate_abandoned` need the scheduler (Mapping §3 item 7, not in M1).
- V-04 / V-05: triggers are checked for existence only; agreement supersession mid-period is not restricted.

## CLI surface not built

- **[M2 done]** No `correction` or `review` subcommands. Both kinds validate and can enter through `import-jsonl`.
- No `policy framework` subcommand (priority groups I/II/III stay out of the kernel; owner decision 2).
- `gate` supports one signer per receipt from the CLI; multi-party gates need hand-built payloads.
- **[M2 done]** `rws init` does not bind `mandate_holder` in solo mode; run `role-bind --role mandate_holder` before a handover.

## Runtime not built (per brief §4)

- `apparatus-maintenance` scheduler.
- SQLite store and migrations (JSONL + CAS files only).
- HTTP, MCP, auth tokens, capability grants, leases, network zones.
- Cedar or any policy evaluator.

## Known limits of the file store

- **[M2 done]** Single writer. No file locking; two concurrent CLI processes can race on `HEAD`.
- CAS write and ledger append are not one transaction: a crash between them leaves an orphan blob (harmless; `check` verifies only admitted artefacts).
- `show … | head` panics on a closed pipe (standard Rust `println!` behaviour).
