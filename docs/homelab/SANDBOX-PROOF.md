# Sandbox proof: identities, replica calls, MCP gate (Aparatus #41)

Run 2026-10-01 on the local stock replica. It is now started with the long-run log config: metrics are not written to `replica.log` (`LOG-POLICY.md`).

- Everything was compiled first, with the replica stopped: `scripts/dev-check.sh` gave 102/0, then the `apparatus`, `apparatusd`, `ic-status` example and `ic-install` builds.
- The replica and apparatusd were started only for this run and stopped right after it.
- Keys live in `.ic-local/` (mode 0600, git-ignored). None is in any commit.
- **No key was rotated.**

## Result

| Check | Result |
|---|---|
| Identities | dev `fimlw-…-mae`, agent `iouat-…-7ae`, anker `5rucf-…-5ae` (`anker.key`), anker2 `eersv-…-3qe` (`anker2.key`) |
| Who seals which daily root | **`r7inp` → `anker2.key`** (`eersv-…`). **`qjdve` and `sgymv` → `anker.key`** (`5rucf-…`). Unchanged since the rotation test |
| `/api/v2/status` | healthy, `impl_version d26cd031…`, **certified height 6 063 at the start, 6 076 at the end** |
| Notary count (`rwlgt`) | 3 |
| Daily-root last (`r7inp`) | day 20727, root `0ff7c0a1…`, count 3, `anchor_root null` |
| Daily-root last (`sgymv`) | day 20727, root `d16e6458…`, count 1, `anchor_root null` |
| Ledger mirror last (`qaa6y`) | entry 2. The certificate verifies against the replica root key, and `certified_data` = sha256(last entry) = `ebd908ce…` |
| Non-anker seals `r7inp` | the agent traps (`assertion failed at v1.mo:66`). **`anker.key` also traps on `r7inp`**, because that canister's anker is now `anker2.key` |
| Non-anker passes the gate | the agent's `check` on mcp-gate traps (`assertion failed at v1.mo:69`) |
| MCP allowed read | `tools/call chain_head` returns the head (`… ff95396a… 322`). Recorded in mcp-gate as **event 4, Allow** |
| **MCP refused call** | `tools/call install_code` returns `refused by mcp-gate: unknown tool: default refuse (args sha256 2b8bc1cb…)`. Recorded in mcp-gate as **event 5, Refuse**, args `2b8bc1cb7e5bf3b847cb56dfd887e822938e1de689c724a0888fc70b4a2381eb`, caller anker `5rucf-…` |

**The refusal Event id is 5** on mcp-gate `sbzkb-zqaaa-aaaaa-aaaiq-cai`. Its args hash matches the hash in the bridge's reply.

## Transcript

```text
$ scripts/ic-up.sh
metrics: file exporter, dumped once at shutdown to run/metrics-at-shutdown.prom (not in replica.log)
log-cap: 0 bytes < cap 67108864; nothing to close
replica up (pid 2245): http://127.0.0.1:8080/api/v2/status
$ scripts/dev-up.sh
socket: /home/user/Aparatus/.apparatus/apparatusd.sock
# identities (no rotation)
$ [dev] ic-install ident   (600 .ic-local/dev.key)
principal fimlw-w22xm-mqzu6-2pemp-yue5t-gf4re-et7uy-mcg6m-7twom-yvcg7-mae
$ [agent] ic-install ident   (600 .ic-local/agent.key)
principal iouat-qhdx3-paugj-jqpva-pr2fp-xowan-d65pt-lmjvt-fqyuk-vkga3-7ae
$ [anker] ic-install ident   (600 .ic-local/anker.key)
principal 5rucf-2grbz-ist7p-etfu5-uf6pv-hwcjb-ffz5f-x627x-wnt32-sma6d-5ae
$ [anker2] ic-install ident   (600 .ic-local/anker2.key)
principal eersv-76f6a-mqovg-e2wbv-gzf6d-jnrnx-z6jmh-uvdvi-3urjz-rixzl-3qe
# which key seals which daily root
$ [dev] ic-install dr-anker r7inp-6aaaa-aaaaa-aaabq-cai
anker Some("eersv-76f6a-mqovg-e2wbv-gzf6d-jnrnx-z6jmh-uvdvi-3urjz-rixzl-3qe")
exit 0
$ [dev] ic-install dr-anker qjdve-lqaaa-aaaaa-aaaeq-cai
anker Some("5rucf-2grbz-ist7p-etfu5-uf6pv-hwcjb-ffz5f-x627x-wnt32-sma6d-5ae")
exit 0
$ [dev] ic-install dr-anker sgymv-uiaaa-aaaaa-aaaia-cai
anker Some("5rucf-2grbz-ist7p-etfu5-uf6pv-hwcjb-ffz5f-x627x-wnt32-sma6d-5ae")
exit 0
# replica calls
$ target/debug/examples/ic-status
replica_health_status healthy
certified_height 6063
impl_version d26cd031176beec51b39fbb9e39e80a3a46a748e
$ [dev] ic-install count rwlgt-iiaaa-aaaaa-aaaaa-cai
count 3
exit 0
$ [dev] ic-install dr-last r7inp-6aaaa-aaaaa-aaabq-cai
day 20_727 root 0ff7c0a1a1e7b5a356be2f274f126d96866dea42329c58b0e510d8ab1161bf1e count 3 at 1_790_875_452_989_505_287 anchor_root null
exit 0
$ [dev] ic-install dr-last sgymv-uiaaa-aaaaa-aaaia-cai
day 20_727 root d16e645867271f7aec65a095b34c48ff6aecd0dca0e278f4b2198b301c38aff2 count 1 at 1_790_877_472_321_745_510 anchor_root null
exit 0
$ [dev] ic-install lm-certified qaa6y-5yaaa-aaaaa-aaafa-cai
last entry 2 01a0f800-df8b-70f8-a3c7-67abd83147f3 head d16e645867271f7aec65a095b34c48ff6aecd0dca0e278f4b2198b301c38aff2
certificate verified against the replica root key
certified_data   ebd908cec81a31e97e22eb6051142356bd9c73abeb5a0c7f46cd2fd36234072d
sha256(entry)    ebd908cec81a31e97e22eb6051142356bd9c73abeb5a0c7f46cd2fd36234072d
certified_data == sha256(last entry)
exit 0
# a principal that is not the anker cannot seal and cannot pass the gate
$ [agent] ic-install dr-seal r7inp-6aaaa-aaaaa-aaabq-cai 20727 0ff7c0a1a1e7b5a356be2f274f126d96866dea42329c58b0e510d8ab1161bf1e 3
Error: The replica returned a rejection error: reject code CanisterError, reject message Error from Canister r7inp-6aaaa-aaaaa-aaabq-cai: Canister called `ic0.trap` with message: 'assertion failed at v1.mo:66.17-66.39'.
exit 1
$ [anker] ic-install dr-seal r7inp-6aaaa-aaaaa-aaabq-cai 20727 0ff7c0a1a1e7b5a356be2f274f126d96866dea42329c58b0e510d8ab1161bf1e 3
Error: The replica returned a rejection error: reject code CanisterError, reject message Error from Canister r7inp-6aaaa-aaaaa-aaabq-cai: Canister called `ic0.trap` with message: 'assertion failed at v1.mo:66.17-66.39'.
exit 1
$ [agent] ic-install mg-check sbzkb-zqaaa-aaaaa-aaaiq-cai chain_head {}
Error: The replica returned a rejection error: reject code CanisterError, reject message Error from Canister sbzkb-zqaaa-aaaaa-aaaiq-cai: Canister called `ic0.trap` with message: 'assertion failed at v1.mo:69.32-69.54'.
exit 1
# MCP: apparatus mcp serve over stdio (anker key), one allowed read, one refused install_code
$ printf "<4 lines>" | target/debug/apparatus mcp serve --gate sbzkb-zqaaa-aaaaa-aaaiq-cai
>> {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"sandbox-proof","version":"0"}}}
>> {"jsonrpc":"2.0","method":"notifications/initialized"}
>> {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"chain_head","arguments":{}}}
>> {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"install_code","arguments":{"canister":"rwlgt-iiaaa-aaaaa-aaaaa-cai","note":"sandbox proof 2026-10-01"}}}
<< {"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"instructions":"Read-only tools. Every call is checked by the mcp-gate canister and recorded there as an Event.","protocolVersion":"2025-06-18","serverInfo":{"name":"apparatus-mcp","version":"0.1.0"}}}
<< {"id":2,"jsonrpc":"2.0","result":{"content":[{"text":"01a0f8a8-2e87-7492-8870-e5c5d2e04356 ff95396aea36298f1429f9e041e3a3aa9916c99039d49d5290e78eee7397b525 322","type":"text"}],"isError":false}}
<< {"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"refused by mcp-gate: unknown tool: default refuse (args sha256 2b8bc1cb7e5bf3b847cb56dfd887e822938e1de689c724a0888fc70b4a2381eb)","type":"text"}],"isError":true}}
# the Events in mcp-gate (last 3)
$ [dev] ic-install mg-show sbzkb-zqaaa-aaaaa-aaaiq-cai | tail -3
event 3 install_code args b9a73d5b4433141de8d5ada4d45ef2beb08af04260dc62d51869e12bfcf102e3 Refuse by 5rucf-2grbz-ist7p-etfu5-uf6pv-hwcjb-ffz5f-x627x-wnt32-sma6d-5ae : unknown tool: default refuse
event 4 chain_head args 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a Allow by 5rucf-2grbz-ist7p-etfu5-uf6pv-hwcjb-ffz5f-x627x-wnt32-sma6d-5ae : registered; reads apparatusd:head; writes nothing
event 5 install_code args 2b8bc1cb7e5bf3b847cb56dfd887e822938e1de689c724a0888fc70b4a2381eb Refuse by 5rucf-2grbz-ist7p-etfu5-uf6pv-hwcjb-ffz5f-x627x-wnt32-sma6d-5ae : unknown tool: default refuse
# certified height at the end
certified_height 6076
$ scripts/dev-down.sh; scripts/ic-down.sh
apparatusd stopped (pid 2414)
replica stopped (pid 2245)
$ pgrep -a replica || echo "no replica"
no replica or apparatusd
```

## What a Tailscale client will call later, and what works now on loopback

- **Works now, on 127.0.0.1 only:**
  - `apparatus mcp serve` over stdio. The client spawns the bridge as a local process.
  - The bridge talks to apparatusd over its 0600 Unix socket, and to the replica at `127.0.0.1:8080`.
- **Later, over Tailscale (Dex's session, Aparatus #8):**
  - A remote MCP client reaches the anker with Tailscale SSH and runs the same `apparatus mcp serve --gate <id>` on the anker.
  - The tools, the gate checks and the Events stay the same. The replica and the socket are still not opened on the tailnet.
