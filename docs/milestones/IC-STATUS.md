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
