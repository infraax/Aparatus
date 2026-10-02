# Any Linux node: `scripts/node-up.sh`

One script for every Linux machine in the homelab: the Oracle VPS, the anker ThinkPad, a Raspberry Pi, a laptop.

- `scripts/vps-up.sh` still works: it is `node-up.sh --role vps`.
- [VPS.md](VPS.md) keeps the VPS rules and its dry runs. Everything there holds for every role.

## Steps

Always start with a dry run. It changes nothing.

```sh
git clone https://github.com/infraax/Aparatus && git clone https://github.com/infraax/NAP-corpus
cd Aparatus
scripts/node-up.sh --nodes-md ../NAP-corpus/docs/ic/NODES.md --role pi --bootstrap --units
```

Then apply, if the tier and plan are right:

```sh
TS_AUTHKEY=<from the Tailscale admin console> scripts/node-up.sh --nodes-md ../NAP-corpus/docs/ic/NODES.md \
  --role pi --node-id pi-garage --bootstrap --units --apply
```

| Step | Dry run | `--apply` |
|---|---|---|
| Measure: RAM, free disk, arch, IPv6 | yes | yes |
| Tier from `NODES.md` floors (full / core / light / refused) | yes | yes |
| `--bootstrap`: toolchain for the tier (`node-bootstrap.sh`) | prints the commands | runs them (sudo for packages) |
| `--units`: systemd `--user` units (`node-units.sh`) | prints the unit files | writes and enables them (does not start) |
| Node key `~/.apparatus-node/node.key` (0600, never printed) | describes it | creates it, prints the principal |
| Tailscale join | describes it | only if `TS_AUTHKEY` is set |
| Local record `~/.apparatus-node/node.json` (now with `role`) | describes it | writes it (0600) |

## Roles

| Role | Typical machine | Expected tier | What changes |
|---|---|---|---|
| `vps` | Oracle Cloud micro (x86, 1 GB) or Ampere (arm64) | light / core | nothing else |
| `anker` | ThinkPad, x86-64 | **full** | warns if the machine is under the full tier, since the anker runs the replica |
| `pi` | Raspberry Pi 4/5, 64-bit OS | core | a 32-bit OS (`armv7l`) is refused: use the 64-bit image |
| `laptop`, `generic` | anything else | any | nothing else |

The role is a label in `node.json` and in the default node id (`<role>-<hostname>`). The rules are the same for every role.

## Toolchain per tier (`node-bootstrap.sh`)

The script supports apt (Debian, Ubuntu, Raspberry Pi OS), dnf (Fedora, RHEL, Oracle Linux) and pacman (Arch).

| Tier | Installs |
|---|---|
| light | git, curl, ca-certificates, openssl, python3 |
| core | light, plus build tools, pkg-config, OpenSSL headers, and rustup (only if `cargo` is missing) |
| full | core, plus Node.js and npm (for mops/moc, from `package-lock.json`), libunwind and zstd |

`--with-tailscale` adds Tailscale from its own repository (`tailscale.com/install.sh`), only if it is missing.

## Services per tier (`node-units.sh`)

| Tier | Units in `~/.config/systemd/user/` |
|---|---|
| light | none |
| core | `apparatusd.service`: the single writer, socket 0600, `UMask=0077` |
| full | `apparatusd.service` and `ic-replica.service` (`scripts/ic-up.sh` / `ic-down.sh`, localhost only) |

To start them at boot without a login: `sudo loginctl enable-linger $USER`.

## Dry runs (2026-10-02, this container; nothing installed)

| Command | Tier | Plan |
|---|---|---|
| `vps-up.sh`, simulated 1 GiB / 45 GB / x86_64 | light | apt: 5 packages; no units |
| `--role anker`, simulated 32 GiB / 400 GB / x86_64 / IPv6 | full | apt: full set; `apparatusd` and `ic-replica` units |
| `--role pi`, simulated 8 GiB / 60 GB / aarch64 | core | full refused (arm64, disk); apt: core set |
| `--role pi`, simulated armv7l | refused | exit 1: needs the 64-bit image |

## Not tested here

- A real Pi, a real ThinkPad, and dnf/pacman hosts.
- A real `--apply` with `--bootstrap` or `--units`. This container has no systemd user session.
- The Tailscale join (waits on Dex).
