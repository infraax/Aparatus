# M4 — deferred

Open after M4. `docs/notes/M3-DEFERRED.md` and `M2-DEFERRED.md` stay valid.

## Conduits named but not built

- MCP notify hook (and an MCP adapter on the JSONL RPC): implement `Hook`; nothing ships in M4.
- Remote wake (Tailscale or similar) and object-storage upload: `Hook` implementations, later.
- Offsite / encrypted backup (rclone, S3, age): `after_backup` is the attachment point. M4 backs up to a local directory only.
- Prometheus or any scrape endpoint: `on_signal` and `.apparatus/SIGNAL` are the conduit.
- SQLite: the JSONL ledger stays until it is measured to be too slow.

## Pipe

- Night ingest windows / rate limits: the pipe runs whenever the daemon polls. No scheduler crate (no `target_missed`).
- `ingest_enqueue` `content` is UTF-8 only (no base64 for binary); binary goes by `path` or by dropping the file.
- Pipeline receipts are signed by the project's default principal, not by the agent that dropped the file (no service or dropper principal; the sidecar carries no identity).
- C-03 `quarantine` and `reference_only` outcomes are not written as receipts: quarantine is a folder plus a ticket, because the bytes may be unreadable.
- C-07 `occurred_at` is not taken from the sidecar.
- A sidecar without its file waits in `in/new/` forever; nothing cleans it up.
- `ok/`, `refuse/` and `quarantine/` grow without rotation.
- Quarantined files are not re-offered automatically: move them back to `in/new/` after fixing.

## Signals

- Every tick runs a full `check`, which re-hashes every CAS blob. Fine for a small corpus; a large one needs sampled or incremental verification.
- Signal tickets close automatically when the cause is gone; `apparatusd: … failing|panicked` tickets and quarantine tickets are closed by hand.
- Chain-integrity failure: the tick cannot append its ticket to a chain that does not verify; it logs and writes SIGNAL only.
- `SIGNAL` is overwritten, not historised (telemetry batches as A-tagged artefacts, T-01, are not generated).

## Backup

- Backups are full copies; no incremental or deduplicated backups, no retention policy.
- Restore does not record `restore_tested`; the owner files that event after a drill.
- The CAS `ids/` pointers of orphans are copied along; they are harmless but not pruned.
