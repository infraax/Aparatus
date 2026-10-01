# Tailnet reads: what to run when the homelab and the Oracle VPS are up

Prepared 2026-10-01 for when Dex hands over Tailscale. **Every step here is a read**, or a report a node makes about itself. No step opens a public port or binds the replica to the tailnet (NAP #27). No step needs a cloud password.

Names below are placeholders until MagicDNS names exist:
- `anker` is the homelab laptop;
- `oracle-micro` is the VPS.

## 0. Dex does these (needs-dex)

1. Turn on the anker. `scripts/ic-up.sh` and `scripts/dev-up.sh` from tag `homelab-0` (Aparatus #35).
2. Create a Tailscale auth key in the admin console. It goes only into the VPS environment as `TS_AUTHKEY`, never into a commit or a chat.
3. On the VPS: clone both repos, then run the paste in `docs/homelab/VPS.md`. Run the dry run first, then `--apply`.

## 1. Reads from the session, over Tailscale SSH

| Read | Command | Answers |
|---|---|---|
| Anker replica health | `tailscale ssh anker -- curl -s http://127.0.0.1:8080/api/v2/status \| head -c 400` | NAP #27: status through the tailnet while the replica binds 127.0.0.1 only |
| Anker head | `tailscale ssh anker -- target/debug/rws check` | Aparatus #24 and #27: same head after restart |
| VPS node record | `tailscale ssh oracle-micro -- cat ~/.apparatus-node/node.json` | Aparatus #39: tier, principal, specs; the anker pulls, nothing is pushed |
| VPS dry run | `tailscale ssh oracle-micro -- Aparatus/scripts/vps-up.sh --nodes-md NAP-corpus/docs/ic/NODES.md` | prints `light` on the AMD micro; prints `core` or exits on Ampere |
| Disk and RAM on the anker | `tailscale ssh anker -- sh -c 'free -b; df -B1 ~'` | NODES.md: is the anker really `full` (≥ 100 GB free) |

The `node.json` and the `node.key` stay on the VPS. Only the principal string moves.

## 2. Writes that need a vote (on the anker's local replica)

1. **Register both nodes in node-status.** Use one advice, a `pb-propose-call` to `registerNode(id, principal, location)`, a vote, and `enact`:
   - `anker`, with the anker principal (`5rucf…`), location `home`;
   - `oracle-micro`, with the principal printed by `vps-up.sh --apply`, location `oracle`.
2. **Each node reports itself.**
   - The anker runs `ic-install ns-report-self <node-status> anker ~ -`.
   - The VPS gets no replica and no IC key on the hot path. The anker relays its `node.json` numbers with `ns-report` signed by the VPS principal's key, which stays on the VPS. Until a light-node CLI exists on the VPS, this is the one step that waits.
3. **Read back.** `ns-describe <node-status> oracle-micro` as a granted reader, and `node_status` through the MCP bridge.

## 3. What stays out

- No Oracle API key, SSH key or cloud password in any file, chat or commit. Real credentials go into the access-keys vault as sealed items. The node record points at the item id only (NAP #50).
- No mainnet deploy, no cycles.
- No replica on the VPS.
- No anker key rotation.
