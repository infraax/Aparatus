# Envelope Stage 0 — notes and deferred

## Decisions the owner should confirm

- **Twelfth receipt kind.** `Agents.md` and RWS-2.0-MAPPING §2 say "11 kinds; adding a kind changes RWS 2.0 itself".
  `envelope` was added on the owner's brief (#12), inside the MAPPING's "~12" limit. `Agents.md` now says twelve.
  RWS-2.0 X-02 in `infraax/delta` is not edited from here.
- **Name collision.** In RWS 2.0 "envelope" already means a means envelope (O-09: `policy envelope`, `envelope_ref`).
  The new kind is wire-named `envelope` as the brief asks; the Rust type is `EventEnvelope`.
  If the kind should be `event_envelope`, rename **before** any real chain carries it: after that, a rename breaks replay.
- **Payload canonical form.** See disagreement 1. CID codec is `raw` (0x55) because the bytes are canonical JSON, not CBOR.

## Disagreements: ENVELOP.md vs T6 (ENVELOP.md followed, no third schema)

1. Canonical encoding — ENVELOP.md §2: deterministic CBOR is canonical. T6 D.1: JCS (RFC 8785) primary, CBOR optional mirror.
   Stage 0 uses neither in full: it uses the chain's existing canonical JSON (RWS X-04: sorted keys, no floats, no
   whitespace), so envelope and ledger hash the same bytes. Not RFC 8785-complete (number/escape rules untested).
   Stage 1/2 must pick one; if CBOR, new receipts use codec `dag-cbor` (0x71) and old ones stay `raw`.
2. Lifecycle states — ENVELOP.md §2: four (`draft → signed → sealed → anchored`). T6 C.1: six (+ `superseded`, `archived_cold`). Four built.
3. Time — ENVELOP.md §1: `unix_timestamp` (mandatory, unique). T6 B.2: `timestamp_captured_utc` RFC 3339 + EDTF + precision. ENVELOP followed.
4. Provenance — ENVELOP.md §1: `provenance` = `stated|measured|inferred`. T6 has no such field; its closest is
   `source_type` (`direct_capture|derived|imported_legacy|external_received|synthetic`). ENVELOP followed.
5. Identity CID — ENVELOP.md §2 / T6 B.1: `entry_id` = CID over the whole (signed) entry. Stage 0 `cid` addresses the
   payload only (T6's content address, `cas://blake3/…`). An `entry_id` over the signed envelope is not built.
6. Signature — ENVELOP.md §2 / T6 D.3: hybrid Ed25519 + ML-DSA-65 (`ed25519_mldsa65_hybrid_v1`, tag
   `ARCHIVE-HYBRID-SIG-v1\0`). Stage 0 signs single-scheme with tag `APPARATUS-ENVELOPE-SIG-v0\0`; the tag is a
   Stage 0 choice, bound to the scheme name, so a hybrid scheme can add its own tag without touching v0 receipts.

## Differences from issue #12's text

- "`rws envelope` already present as placeholder": it was not; `rws policy envelope` (means) exists. `rws envelope` is new.
- "Ed25519 implementation (existing)": none existed. Added `ed25519-dalek` 2.x, `blake3` 1.8, `getrandom` 0.4 (already in the lock via uuid).
- "Every ingest produces an envelope receipt": **not built.** It changes `rws ingest` output and receipt counts and
  overlaps the M4 ingest pipe. Deferred to M4 / Stage 1.

## Deferred

- Sealing enforcement, immutability of `sealed`, lifecycle transition receipts (#13).
- CID dedup / lookup by CID; `show <cid>` (#13).
- ML-DSA-65 and hybrid (#14); Bitcoin anchoring (#15).
- `unix_timestamp` uniqueness (ENVELOP.md "uniek"): only `event_id` is unique (ENV-05).
- ENVELOP.md §1 fields not carried: `captured_at`, `local_tz`, `event_time`, `window`, `event_type`, `instrument_version`,
  `item_id`, `cadence_tier`, `mode`, `device`, `channel`, `schema_version`. Additive later (T6 G).
- Payloads > 4 KB via CAS (`content_uri`); today they are refused (ENV-02).
- Keys: one node key per project, plaintext seed at mode 0600; no per-principal keys, rotation, HSM, or backup of `keys/` (M4 backup must decide).
- `corrects` pointer for envelope corrections (ENVELOP.md §1 `payload.corrects`): the existing `correction` kind applies.
- A refused envelope's full attempted payload is copied into the refusal's `detail` (existing M1 behaviour for all
  kinds). An oversized payload (ENV-02) therefore still lands on the chain inside the refusal. Truncate in #13 or M4.
- Replay re-verifies every signature on open: O(n) Ed25519 verifies; measure before optimising.
