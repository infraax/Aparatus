# Envelope Stage 1 — lifecycle, provenance and content-address rules

NAP-corpus issue #13. Builds on Stage 0 (`ENVELOPE-STAGE-0.md`) without changing its shape. Receipt kind stays
`event_envelope`. All rules are schema rules in `State::check`; a violation is an `illegal_transition_rejected`
receipt on the chain and exit 2 (same pattern as K-01).

## Rules added

| Rule | What |
|---|---|
| ENV-05 | Per `event_id`: `draft` → `draft` \| `signed`; `signed` → `sealed` only. `signed` is one-way. `draft` → `sealed` is refused. Sealing changes nothing but `lifecycle`. |
| ENV-06 | `measured` needs `evidence_tag` (A, B, C, E, NF). `inferred` needs a non-empty `source_ref`. `stated` needs neither. |
| ENV-07 | A `sealed` `event_id` never changes again: any later receipt for it is refused. |
| ENV-08 | `anchored` is refused. Anchoring is Stage 3 (#15). |
| ENV-09 | One CID, one content object. See below. |

- `draft` → `signed` needs a signature that verifies (ENV-04). Ed25519 is checked for real; the ML-DSA-65 stub
  (`ml_dsa_65_stub`) still verifies as `true`. No ML-DSA implementation.
- A new `event_id` may start at `draft`, `signed` or `sealed`. Sealed-at-creation still needs a valid signature.
- New optional fields `evidence_tag`, `source_ref`, `duplicate_of`: omitted from the receipt when absent, so Stage 0
  receipts keep their exact bytes and signatures.

## Duplicate payloads: stored as a reference (choice)

CID v1 + BLAKE3 stays mandatory on every `event_envelope` (ENV-03). When the same canonical payload bytes arrive a
second time under another `event_id`:

- they get the **same CID**;
- the second receipt is **accepted as a reference**: the writer sets `duplicate_of = <event_id that first carried this CID>`
  before signing, so the reference is inside the signature scope;
- the chain keeps **one owner per CID** (`State::cid_owner`); the reference never replaces it;
- a missing or wrong `duplicate_of`, or a `duplicate_of` on a CID nobody carries yet, is refused (ENV-09) on the chain.

Why a reference, not a refusal: two distinct events (different source, time, provenance) can carry identical bytes,
e.g. two sensors reporting the same reading. Refusing would drop the second event; a reference keeps it and still
gives one content object. Nothing is dropped silently. The same `event_id` moving through its lifecycle is not a duplicate of itself.
The payload stays inline in the reference receipt (≤ 4 KB); moving bytes to CAS is still deferred.

## Not built

- ML-DSA-65 / hybrid (#14); Bitcoin anchoring (#15).
- Replication check behind `sealed` (ENVELOP.md "≥ 2 archive locations"): `sealed` is a declared, one-way state, not a verified one.
- `rws ingest` is unchanged; ingest emitting `event_envelope` stays in the M4 pipe.
- No IC replica wiring.
