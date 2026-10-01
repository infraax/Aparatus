# IC status client

`crates/apparatus-ic`: a read-only client for a local IC replica, built on the official agent crate.

- Agent: **`ic-agent` 0.49.2** (dfinity/agent-rs), pinned `=0.49.2`, `default-features = false`. Async calls run on a
  current-thread `tokio` runtime inside one blocking function.
- Call: `Agent::status()` → `GET <url>/api/v2/status` (CBOR). No identity, no canister call, no root-key fetch for queries.
- `apparatus_ic::status(url) -> Result<ReplicaStatus, IcError>`. It never starts or stops a replica.
- Transport: `reqwest` client from `ic_agent::export`, 5 s timeout, `with_max_tcp_error_retries(0)`, so a down replica fails fast.

| Field | Source |
|---|---|
| `health` | `Status.replica_health_status` |
| `certified_height` | `Status.values["certified_height"]` (the agent has no typed field for it) |
| `impl_version` | `Status.impl_version` |
| `root_key_sha256` | SHA-256 of `Status.root_key` (DER) |
| `root_key_principal` | `Principal::self_authenticating(root_key)`. **Not exposed by the agent**; derived here. On this root subnet it equals the subnet id the replica logs |

Errors (`IcError`): `Unreachable` (agent `TransportError`: refused, timeout), `BadResponse` (answered, not a valid status),
`InvalidUrl`.

## Proof against the replica from `scripts/ic-up.sh` (2026-10-01)

```text
$ scripts/ic-up.sh
binaries already verified against SHA256SUMS; no download
replica up (pid 5794): http://127.0.0.1:8080/api/v2/status
$ cargo run -q -p apparatus-ic --example ic-status
ic-agent 0.49.2
url http://127.0.0.1:8080/api/v2/status
replica_health_status healthy
certified_height 18
impl_version d26cd031176beec51b39fbb9e39e80a3a46a748e
root_key_sha256 ac826e4f1169d4e2ad25092e9fb34c25037510f3089b4d0864bc663c7e46ac15
root_key_principal av6td-ktmkn-fzsqe-7czvi-ujd6x-hdf2k-mruxw-av2uj-2xgc7-ozaqs-qqe
```

Replica log subnet id: `s:av6td-ktmkn-fzsqe-7czvi-ujd6x-hdf2k-mruxw-av2uj-2xgc7-ozaqs-qqe`, the same as `root_key_principal`.

Down (nothing on port 8081):

```text
$ cargo run -q -p apparatus-ic --example ic-status http://127.0.0.1:8081
error: replica unreachable at http://127.0.0.1:8081: error sending request for url (http://127.0.0.1:8081/api/v2/status)
exit 3
```

## Tests (no replica needed)

- `down_replica_is_a_typed_unreachable_error`: closed port → `IcError::Unreachable`.
- `reads_health_height_and_root_key_through_ic_agent`: a local HTTP stub serves hand-encoded CBOR (self-describe tag,
  `replica_health_status`, `certified_height`, `root_key`, `impl_version`); the agent parses it.
- `non_status_body_is_a_bad_response_not_unreachable`.
- `status_url_joins_the_path`.

## `rws ic status`: the replica height on the chain

```bash
apparatus rws ic status [--url http://127.0.0.1:8080]
```

- The **client** reads the replica through `apparatus_ic::status` (ic-agent). The **writer** (apparatusd, or the CLI
  under LOCK) records it. The daemon makes no network call; its single writer is never blocked on HTTP.
- Healthy read → one `event_envelope`: `provenance measured`, `evidence_tag B`, `source` = the status URL,
  `module apparatus.ic.status`, signed by the writer. Payload is only
  `{"certified_height": <n>, "replica_health_status": "<s>"}`, not the whole status body.
- Same height again → same CID → the writer sets `duplicate_of` (ENV-09): one content object, the read is still recorded.
- Replica down (agent transport error) → `illegal_transition_rejected` receipt, rule **IC-01**, exit 2. Something answered
  but not with a valid status → **IC-02**, exit 2. Never dropped. A bad `--url` is a usage error (exit 1, no receipt).
- RPC: `{"op":"ic_status","url":"…/api/v2/status","reading":{"result":"ok","replica_health_status":"healthy","certified_height":520}}`
  (`"result":"unreachable"|"bad_response"` with `reason`).

### Daemon-path demo (2026-10-01, fresh `.apparatus/`, replica from `scripts/ic-up.sh`)

```text
$ scripts/ic-up.sh
binaries already verified against SHA256SUMS; no download
replica up (pid 14200): http://127.0.0.1:8080/api/v2/status
exit 0
$ scripts/dev-up.sh
initialised solo project in /home/user/Aparatus/.apparatus
apparatusd pid 14369, log /home/user/Aparatus/.apparatus/apparatusd.log
socket: /home/user/Aparatus/.apparatus/apparatusd.sock
exit 0
# LOCK held by apparatusd: the rws calls below go over the socket
$ target/debug/apparatus rws ic status
ic status http://127.0.0.1:8080/api/v2/status healthy certified_height 520
event_envelope 01a0f7f0-b97b-7141-96ab-182cb6395a81 cid bafkr4ihzxoaw6xj4vzuyd3uusewvuour745quchlrnecti5daznlgozi2u
01a0f7f0-b97d-75a1-a800-8e0e93609dd7 event_envelope 4f2f93e9572e320438d8e0bef436c3cce99979dd7effcfd1527740885b31ef00
exit 0
$ target/debug/apparatus rws ic status & target/debug/apparatus rws ic status   # same moment
ic status http://127.0.0.1:8080/api/v2/status healthy certified_height 520
event_envelope 01a0f7f0-b9ad-728f-9e4f-5d87764a927b cid bafkr4ihzxoaw6xj4vzuyd3uusewvuour745quchlrnecti5daznlgozi2u duplicate_of 01a0f7f0-b97b-7141-96ab-182cb6395a81
01a0f7f0-b9ae-7530-bd93-e5afe71eb3bf event_envelope b4ac21cc48d0b668baa29adf991772d591856b1b59fde110df345fef38110d77
ic status http://127.0.0.1:8080/api/v2/status healthy certified_height 520
event_envelope 01a0f7f0-b9ba-721e-ab72-4d39af53bd27 cid bafkr4ihzxoaw6xj4vzuyd3uusewvuour745quchlrnecti5daznlgozi2u duplicate_of 01a0f7f0-b97b-7141-96ab-182cb6395a81
01a0f7f0-b9ba-721e-ab72-4d3ab7c72be4 event_envelope f52e19b1c9c295ee92a580bda36a0c77a65ca42a8beea801edf46af5d82aa3d3
$ scripts/ic-down.sh
replica stopped (pid 14200)
exit 0
$ target/debug/apparatus rws ic status
01a0f7f0-c1d7-7237-85b3-2357b35bd5b7 event 975fab240e3ded0684caf6f368d2bc3a443e5edb2ca167b01ae01bc8adac3e18
REFUSED IC-01: replica unreachable at http://127.0.0.1:8080/api/v2/status: error sending request for url (http://127.0.0.1:8080/api/v2/status)
exit 2
$ target/debug/apparatus rws head
01a0f7f0-c1d7-7237-85b3-2357b35bd5b7 975fab240e3ded0684caf6f368d2bc3a443e5edb2ca167b01ae01bc8adac3e18 8
exit 0
$ scripts/dev-down.sh
apparatusd stopped (pid 14369)
exit 0
$ scripts/dev-up.sh
apparatusd pid 14426, log /home/user/Aparatus/.apparatus/apparatusd.log
socket: /home/user/Aparatus/.apparatus/apparatusd.sock
exit 0
$ target/debug/apparatus rws head
01a0f7f0-c1d7-7237-85b3-2357b35bd5b7 975fab240e3ded0684caf6f368d2bc3a443e5edb2ca167b01ae01bc8adac3e18 8
exit 0
$ target/debug/apparatus rws check
project Aparatus (01a0f7f0-b876-7190-a8f6-2cc64b7925bc)
chain ok: 8 receipts, head 01a0f7f0-c1d7-7237-85b3-2357b35bd5b7 975fab240e3ded0684caf6f368d2bc3a443e5edb2ca167b01ae01bc8adac3e18
cas ok: 0 artefacts verified
role continuity: owner
role policy: owner
role execution: owner
role mandate_holder: owner
refusals recorded: 1
  refused 01a0f7f0-c1d7-7237-85b3-2357b35bd5b7 event_envelope refused: IC-01: replica unreachable at http://127.0.0.1:8080/api/v2/status: error sending request for url (http://127.0.0.1:8080/api/v2/status)
open failures: 0
open tickets: 0
check ok
exit 0
$ scripts/dev-down.sh
apparatusd stopped (pid 14426)
exit 0
```

The two calls started at the same moment both read height 520 and were serialized by the one writer: both are
`duplicate_of` the first read. **Height on the receipt: 520.** **Head after the apparatusd restart:**
`01a0f7f0-c1d7-7237-85b3-2357b35bd5b7 975fab240e3ded0684caf6f368d2bc3a443e5edb2ca167b01ae01bc8adac3e18` (8 receipts),
the same as before the restart; `rws check` → `check ok`. The replica resumed at 520 because it restarted from its
height-500 checkpoint in `.ic-local/run`.

Tests: CLI `ic_status_records_height_once_and_refuses_when_down` (HTTP stub at a fixed height: second read is
`duplicate_of`; closed port → IC-01, exit 2); daemon `ic_status_over_rpc_duplicate_and_refusals` (duplicate, new
height, IC-01, IC-02 over the socket). 85 → 87.
