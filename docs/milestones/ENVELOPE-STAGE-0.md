# Envelope Stage 0 — typed envelope on the existing ledger

Receipt kind, RPC op and CLI are named `event_envelope` / `rws event-envelope`. `envelope` alone is the RWS means
envelope (`rws policy envelope`, `envelope_ref`) and is not a receipt kind.

NAP-corpus issue #12. A layer on the M1–M3 chain: same `.apparatus/` files, same single writer,
same SHA-256 link. Not a new ledger, not a replacement for the RWS receipt.

```text
rws event-envelope / RPC op "event_envelope" → apparatusd (or local under LOCK) → payload_hash + cid + sign → State::check (ENV-01..05) → ledger
```

## What landed

| Piece | Where |
|---|---|
| Receipt kind `event_envelope` (`Body::EventEnvelope(EventEnvelope)`) | `crates/apparatus-types/src/rws.rs` |
| `ProvenanceKind` enum: `stated` / `measured` / `inferred` | same |
| `Lifecycle` enum: `draft` / `signed` / `sealed` / `anchored` (stored, not enforced) | same |
| `SignatureScheme` enum: `ed25519`, `ml_dsa_65_stub` | same |
| `Blake3Digest`, `Cid` (v1, raw/dag-cbor codec, BLAKE3-256 multihash, base32 `b…`) | `crates/apparatus-crypto/src/cid.rs` |
| `SigningKey` trait, `Ed25519Key`, `MlDsa65Stub`, `key_id`, framing | `crates/apparatus-crypto/src/signing.rs` |
| Canonical JSON moved to crypto (ledger re-exports it; same bytes) | `crates/apparatus-crypto/src/canonical.rs` |
| Envelope rules ENV-01..04, signing bytes, sign/verify | `crates/apparatus-schema/src/envelope.rs` |
| ENV-05 (`event_id` unique on chain) | `State::check_envelope` in `crates/apparatus-schema/src/rws.rs` |
| `Op::Envelope`, node key `.apparatus/keys/ed25519.seed` (0600, dir 0700) | `crates/apparatus-kernel/src/{api,kernel}.rs` |
| `apparatus rws event-envelope` | `bins/apparatus-cli/src/main.rs` |

## Commands

```bash
apparatus rws event-envelope --payload reading.json --source sensor:bme280 --module dexos.climate --provenance measured
apparatus rws event-envelope --data '{"n":1}' --source me --module dexos.notes --provenance stated --scheme ml-dsa-65-stub
apparatus rws event-envelope --data '{"n":1}' --source me --module dexos.notes --provenance inferred --lifecycle draft
echo '{"n":2}' | apparatus rws event-envelope --payload - --source me --module dexos.notes --provenance stated --event-id evt-2
```

Output: `event_envelope <event_id> cid <cid>` then `<receipt-id> event_envelope <hash>`. Exit 0 ok, 2 refused (refusal on the chain), 1 error.
RPC: `{"op":"event_envelope","payload":{…},"source":"…","module":"…","provenance":"measured"}`; optional
`event_id`, `unix_timestamp`, `lifecycle`, `signature_scheme`, and client-stated `payload_hash`/`cid` (checked, ENV-03).

## Field mapping (brief → chain)

| Brief / #12 | On the chain | Source of the name |
|---|---|---|
| `event_id` | `event_id` | ENVELOP.md §1 |
| `unix_ts` | `unix_timestamp` (seconds) | ENVELOP.md §1 (renamed per the brief's rule) |
| `source`, `module` | `source`, `module` | ENVELOP.md §1 |
| `provenance` | `provenance` (`ProvenanceKind`) | ENVELOP.md §1 |
| `payload_hash` | `payload_hash` (hex SHA-256 of canonical payload) | #12; T6 calls its analogue `content_hash` |
| `lifecycle` | `lifecycle` | #12; ENVELOP.md §2 names no field; T6 calls it `state` |
| `cid` | `cid` (CID v1 raw/BLAKE3 of canonical payload) | #12; not ENVELOP's `entry_id` (see NOTES) |
| `signature_scheme` | `signature_scheme` | #12; T6 calls it `signature_algorithm` |
| — | `payload` (inline JSON, ≤ 4 KB) | ENVELOP.md §1 `payload`; T6 B.4 inline limit |
| — | `signature {signing_key_id, public_key, value}` | T6 B.9 / D.3 |

## Rules (refusals are `illegal_transition_rejected` receipts)

- ENV-01 `event_id`, `source`, `module` non-empty; `unix_timestamp` > 0.
- ENV-02 payload is canonical JSON (no floats) and ≤ 4096 bytes.
- ENV-03 `payload_hash` and `cid` match the payload.
- ENV-04 `draft` has no signature; `signed`/`sealed`/`anchored` have one that verifies under `signature_scheme`.
- ENV-05 `event_id` is not already on the chain.

Signing bytes: `"APPARATUS-ENVELOPE-SIG-v0\0" || u32_be(len) || canonical_json(envelope − {signature, lifecycle})`.
`signature_scheme` is inside the scope; `key_id = "ARCHIVE-KID-" || base32(SHA-256(scheme‖0‖pk)[0..10])`.
Replay (`Kernel::open`, daemon start, `check`) re-verifies every envelope signature.

## Stubbed on purpose

- `ml_dsa_65_stub`: `verify()` returns `true` for anything; its "signature" is BLAKE3(pk‖msg). It authenticates nothing.
  Stage 2 (#14) adds a **new** scheme name (`ml_dsa_65`, hybrid); it must not change the stub, so old receipts replay as they were.
- `sealed` / `anchored` are accepted as declared. No replication check, no immutability rule (#13), no anchor (#15).
- CID is computed and stored; nothing deduplicates on it (#13).

## Tests added (54 → 71)

crypto 8 (BLAKE3 vector, RFC 4648 base32, CID layout vs independent Python, RFC 8032 vector 1, tamper, stub-true, key id, framing) ·
types 1 (wire names, closed enums) · schema 5 (sign/validate, tamper → ENV-04/03, stub, every ENV rule, key order) ·
CLI 2 (signed/stub/draft/sealed + key mode + replay; five refusals on chain) · daemon 1 (RPC event_envelope, wrong CID refused on chain).
The old kind name `envelope` is refused at decode (types test).

Daemon: `Job::Request` now boxes the request (clippy `large_enum_variant` after `Op::Envelope`). No behaviour change.

Open items and spec disagreements: `docs/notes/ENVELOPE-STAGE-0-NOTES.md`.
