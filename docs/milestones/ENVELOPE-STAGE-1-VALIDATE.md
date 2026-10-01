# Envelope Stage 1 — validation run

Validation of `claude/envelope-stage-1` @ `c83a382` as a closed section. No rule was added or changed.
Run 2026-10-01 in the session container (Linux, IPv4 only). Binaries from `cargo build --workspace --locked`.

## Part 1 — fresh gates

From `cargo clean`:

| Gate | Result |
|---|---|
| `cargo test --workspace` | 81 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `cargo build --locked` (and `--workspace --locked`) | clean |

## Part 2 — one daemon run, through the socket

Project `$P=/home/user/apv` (short path: Unix socket paths are limited to ~100 bytes).

```bash
apparatus --project $P rws init --solo
apparatusd --project $P &                          # socket $P/.apparatus/apparatusd.sock, mode 600
apparatus --project $P rws event-envelope --data '{"temp_c":21,"room":"lab"}' --source sensor:bme280 \
  --module dexos.climate --provenance measured --evidence-tag A --event-id val-1 --lifecycle draft
apparatus --project $P rws event-envelope --event-id val-1 --to signed
apparatus --project $P rws event-envelope --event-id val-1 --to sealed
apparatus --project $P rws event-envelope --data '{"room":"lab","temp_c":21}' --source sensor:bme281 \
  --module dexos.climate --provenance measured --evidence-tag B --event-id val-2
apparatus --project $P rws event-envelope --event-id val-1 --data '{"temp_c":22,"room":"lab"}' \
  --source sensor:bme280 --module dexos.climate --provenance measured --evidence-tag A     # exit 2
apparatus --project $P rws head
kill -TERM <apparatusd pid>                         # exit 0, socket removed
apparatusd --project $P &
apparatus --project $P rws head
apparatus --project $P rws check
```

Output (abridged to the envelope lines):

```text
event_envelope val-1 cid bafkr4iam62yjm4esdwfcwaumz7x7cjtt2d47rbzdmahphy5cyi4c4s6s7q
event_envelope val-1 draft -> signed
event_envelope val-1 signed -> sealed
event_envelope val-2 cid bafkr4iam62yjm4esdwfcwaumz7x7cjtt2d47rbzdmahphy5cyi4c4s6s7q duplicate_of val-1
01a0f7d0-c852-70c8-b8fa-28ee8fffc876 event 55ab712f937add99a08f1f72fa4e102567767e217f195e03f507844d04df591c
REFUSED ENV-07: event_id val-1 is sealed; it cannot change          (exit 2)
```

On the chain (`ledger.jsonl`, envelope and refusal receipts):

| Receipt | event_id | lifecycle | duplicate_of | note |
|---|---|---|---|---|
| `01a0f7d0-c79c-…` | val-1 | draft | — | evidence_tag A |
| `01a0f7d0-c7cb-…` | val-1 | signed | — | signed by apparatusd |
| `01a0f7d0-c7e7-…` | val-1 | sealed | — | same signature |
| `01a0f7d0-c816-…` | val-2 | signed | **val-1** | same CID, key order differed |
| `01a0f7d0-c852-…` | — | — | — | `illegal_transition_rejected`: ENV-07 val-1 sealed |

Socket, not local mode: while apparatusd ran, a non-blocking `flock` on `$P/.apparatus/LOCK` failed (held by the
daemon). Local mode needs that lock, so every CLI call above went over the socket.

**Head before stop and after restart (identical):**
`01a0f7d0-c852-70c8-b8fa-28ee8fffc876 55ab712f937add99a08f1f72fa4e102567767e217f195e03f507844d04df591c`, 9 receipts.
After restart, `rws check`: `chain ok: 9 receipts`, `refusals recorded: 1`, `check ok`.
Daemon log: `listening … (4 receipts)` → stop → `listening … (9 receipts)` → stop; both stops exit 0.

## Part 3 — ENV-09 on the paths the CLI cannot reach

The writer always sets `duplicate_of`, so ENV-09 is only reachable by receipts that bypass it.
Crafted line: a copy of a real `draft` envelope `imp-1` (so no signature is involved), with `event_id: "imp-2"`,
a new receipt id, the same payload/CID, and `duplicate_of: "nobody"`.

```bash
S=/home/user/apv-imp
apparatus --project $S-src rws init --solo
apparatus --project $S-src rws event-envelope --data '{"temp_c":21}' --source sensor:bme280 --module dexos.climate \
  --provenance measured --evidence-tag A --event-id imp-1 --lifecycle draft
# $S/rws/receipts.jsonl = the 5 lines of $S-src/.apparatus/ledger.jsonl + the crafted line 6
#   (previous_receipt_id/previous_hash null, so import fills the link)
apparatus --project $S rws import-jsonl $S/rws/receipts.jsonl
```

```text
error: refused: line 6: ENV-09: cid bafkr4iew3jsqb5br6pu65svhs3aerz76bxbx6kvvopbxi326gjz2whgefi is already carried by event_id imp-1; set duplicate_of to it
exit 1
```

**Finding: the refusal is not written on a chain on this path, by design.** `import-jsonl` imports only into an empty
chain and is all-or-nothing (X-11): it dry-runs every line first and refuses the whole buffer before any append.
Result: exit 1, no `ledger.jsonl`, buffer not frozen. ENV-09 fired; there is no chain to record it on. Changing that
would extend the import rules, which this validation does not do.
Side effect seen (pre-existing, not Stage 1): the refused import leaves `.apparatus/project.json`, `LOCK` and `cas/`
in the target directory. A later import into it then fails on the empty-chain check, not on ENV-09.

Replay path, same crafted line appended directly to a copy of `$S-src` with a correct `previous_hash` and `HEAD`:

```text
$ apparatus --project $S rws check
FAIL chain: integrity: line 6 replays as illegal: ENV-09: cid bafkr4iew3jsqb5br6pu65svhs3aerz76bxbx6kvvopbxi326gjz2whgefi is already carried by event_id imp-1; set duplicate_of to it
exit 1
```

On replay ENV-09 makes the chain fail integrity: `Kernel::open` refuses to load it, so the daemon will not start on it.
