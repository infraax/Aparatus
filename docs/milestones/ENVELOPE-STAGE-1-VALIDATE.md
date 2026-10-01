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
in the target directory. Checked: importing the same buffer again refuses on ENV-09 again (exit 1); importing the
5 good lines alone then succeeds (exit 0, buffer frozen). The leftover files do not block a correct import.

Replay path, same crafted line appended directly to a copy of `$S-src` with a correct `previous_hash` and `HEAD`:

```text
$ apparatus --project $S rws check
FAIL chain: integrity: line 6 replays as illegal: ENV-09: cid bafkr4iew3jsqb5br6pu65svhs3aerz76bxbx6kvvopbxi326gjz2whgefi is already carried by event_id imp-1; set duplicate_of to it
exit 1
```

On replay ENV-09 makes the chain fail integrity: `Kernel::open` refuses to load it, so the daemon will not start on it.

## Part 4 — documented refusals

Fresh solo project, local mode. Each refusal is an `illegal_transition_rejected` receipt (the `event` line) and exit 2.

```text
$ apparatus rws event-envelope --data '{"n":1}' --source s --module dexos.neg --provenance stated --event-id neg-a --lifecycle anchored
  01a0f7d2-5c7c-74ee-8b94-ae36f2e4be78 event 5e73ea33d1281f10d0e4e2959fd89e8428ad486a31b83e27cfe6136f6eead6de
  REFUSED ENV-08: anchoring is Stage 3; an envelope cannot be anchored yet
  exit 2
$ apparatus rws event-envelope --data '{"n":2}' --source agent:planner --module dexos.neg --provenance inferred
  01a0f7d2-5ca7-71c5-bed8-ec4f0ca54c44 event ff2d5b1c938dd015d216bbbdf9bf12ca73c3de7756af80fd5cfa36c53e5a5051
  REFUSED ENV-06: inferred provenance needs a source_ref
  exit 2
$ apparatus rws event-envelope --data '{"n":3}' --source sensor:x --module dexos.neg --provenance measured
  01a0f7d2-5cd3-72e3-a930-86df36ac53b9 event 4dbb7d5f2931e914d78c51ed0d2e618f6d906d86f25b9b2501938befdeadbe19
  REFUSED ENV-06: measured provenance needs an evidence_tag (A, B, C, E or NF)
  exit 2
$ apparatus rws event-envelope --data '{"n":4}' --source sensor:x --module dexos.neg --provenance stated --event-id neg-s
  … event_envelope neg-s … (signed)    exit 0
$ apparatus rws event-envelope --data '{"n":4}' --source sensor:OTHER --module dexos.neg --provenance stated --event-id neg-s --lifecycle sealed
  01a0f7d2-5d37-7411-b6d4-f2811010adfc event 9be161c77008ea95927341cb3c23b203bd29306ed1bb2428f8dbe1ead1606446
  REFUSED ENV-05: sealing event_id neg-s may change only its lifecycle
  exit 2
```

`rws check` afterwards: `chain ok: 9 receipts` (4 binds, 1 envelope, 4 refusals), `refusals recorded: 4`, `check ok`.

**A clap typo is not a refusal.** It also exits 2, but nothing reaches the kernel:

```text
$ apparatus rws event-envelope --data '{"n":5}' --source s --module dexos.neg --provenance stated --lifecycle frozen
  error: invalid value 'frozen' for '--lifecycle <LIFECYCLE>': invalid value 'frozen'
  exit 2
```

No `REFUSED` line, no `event` receipt line, chain still 9 receipts with the same head
(`01a0f7d2-5d37-…` `9be161c7…`). Tell the two apart by the `REFUSED <rule>` line on stderr, not by the exit code.

## Not run

- A second daemon or concurrent clients during the walk (covered by M3 tests, not re-run here).
- ENV-04 forgery through the CLI: the CLI cannot submit a forged signature; covered by the schema test.
- ML-DSA (stub only), anchoring, sealing replication: out of scope by design.
