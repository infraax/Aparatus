# Drop-in VPS install (Aparatus #39)

`scripts/vps-up.sh`. Order: measure → refuse → install plan → keys → Tailscale → local record.

## What Dex pastes

Dry run first. It changes nothing.

```sh
git clone https://github.com/infraax/Aparatus && git clone https://github.com/infraax/NAP-corpus
cd Aparatus && scripts/vps-up.sh --nodes-md ../NAP-corpus/docs/ic/NODES.md
```

Then, if the tier is right:

```sh
TS_AUTHKEY=<from the Tailscale admin console> scripts/vps-up.sh --nodes-md ../NAP-corpus/docs/ic/NODES.md --apply --node-id oracle-micro
```

Without `TS_AUTHKEY`, `--apply` makes the key and the record, then stops before the join.

## Rules the script keeps

- **Floors are read from `NODES.md`.** No file, no install.
- **Tier is computed**, the same rule as the node-status canister:
  - full: x86-64 and at or above both full floors;
  - core: at or above both core floors;
  - light: at or above both light floors;
  - below light: refused, exit 1.
- **Full is never offered under the full floor.** The script prints why (RAM, disk, or arm64) and offers the tier it got.
- **No tier downloads the replica here.** Light and core never run it. Full nodes are installed on the anker path (`scripts/ic-up.sh`, stock binary checked against `SHA256SUMS`).
- **Orchestrator:** never tried without an IPv6 default route (NAP #24). This script does not install it at all.
- **No public port.** The replica stays on localhost (NAP #27).
- **Keys:** the seed is 32 random bytes in `~/.apparatus-node/node.key`, mode 0600, never printed. The script prints only the self-authenticating principal. The derivation was checked against two known keys (dev `fimlw…`, anker `5rucf…`).
- **Tailscale:** joins only if `TS_AUTHKEY` is already set, with `--ssh` and the node id as hostname. A key is never invented.
- **Local record:** `~/.apparatus-node/node.json` (0600). It holds id, tier, principal, RAM, disk, arch, IPv6 and addresses. The anker pulls it over Tailscale SSH and writes it into node-status by a voted `registerNode`. The VPS then reports itself.
- `--simulate-*` works only in a dry run. `--apply` on simulated numbers is refused.

## Dry runs (2026-10-01, this container; no Tailscale, nothing installed)

| Shape | Tier | Notes |
|---|---|---|
| Oracle AMD micro: 1 GiB RAM, 45 GB, x86_64, no IPv6 | **light** | full refused (RAM, disk); replica not downloaded; orchestrator skipped |
| Oracle Ampere: 24 GiB, 150 GB, aarch64, IPv6 | **core** | full refused: arm64 has no stock replica binary |
| 128 MiB | refused | exit 1 |
| This container (measured): 15.7 GiB, 16.4 GB free | **core** | full refused: disk |
| No `NODES.md` | stop | exit 1 |

`--apply` into a scratch directory without `TS_AUTHKEY`: wrote `node.key` and `node.json`, both 0600; printed the principal; stopped before the join. The scratch directory was deleted.

## Not tested here

- A real Debian or Ubuntu VPS, and the arm64 path on real hardware.
- The Tailscale join.

Both wait on Dex: the Oracle VPS and a `TS_AUTHKEY`.
