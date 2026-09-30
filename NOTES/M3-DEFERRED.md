# M3 — deferred

Open after M3. See also `NOTES/M2-DEFERRED.md` (still valid).

- MCP adapter: a thin client on the same JSONL RPC (M4 candidate). No MCP server in M3.
- TCP bind (loopback or Tailscale address): Unix socket only. Remote clients reach the host first (SSH/Tailscale) and use the socket.
- Authentication beyond socket file mode 0600 and bound principal names: no tokens, no per-client credentials. Any process that can open the socket can name any bound principal.
- Parallel readers: reads go through the writer queue (short, in order). Snapshot readers are not built.
- SQLite: the JSONL ledger stays the default until it is measured to be too slow.
- Multi-project in one process: one daemon per `--project`.
- HA / clustering: one node plus restart (systemd or `just daemon`).
- Ingest by `path` reads the daemon host's filesystem; `content` is UTF-8 text only (no base64 for binary).
- Internal-error tickets are filed under the requesting principal; there is no dedicated `apparatusd` service principal.
- Windows: `apparatusd` is Unix-only; the CLI runs locally there.
- A `busy` after the reply timeout does not say whether the write landed; clients must check `head`.
