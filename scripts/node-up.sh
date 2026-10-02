#!/usr/bin/env bash
# Drop-in install for any Linux node (Aparatus #39): the Oracle VPS, the anker ThinkPad, a
# Raspberry Pi, a laptop. Order: measure, refuse, toolchain, keys, join, register.
#
#   scripts/node-up.sh --nodes-md PATH [--role vps|anker|pi|laptop|generic] [--apply]
#                      [--bootstrap] [--units] [--node-id ID] [--simulate-ram BYTES]
#                      [--simulate-disk BYTES] [--simulate-arch ARCH] [--simulate-ipv6 0|1]
#
# Default is a DRY RUN: it measures, prints the tier and the plan, and changes nothing.
# --apply generates the node key (0600, seed never printed), writes the local node record,
# and joins Tailscale only if TS_AUTHKEY is already set. It never opens a public port and
# never downloads the replica: full nodes run it on the anker path (ic-up.sh), not here.
# --bootstrap installs the toolchain for the tier (scripts/node-bootstrap.sh, same dry-run rule).
# --units writes systemd --user units for this tier (scripts/node-units.sh, same dry-run rule).
# Floors are read from NAP-corpus docs/ic/NODES.md. If that file is missing, it stops.
set -euo pipefail

NODES_MD="" APPLY=0 ROLE=generic NODE_ID="" BOOTSTRAP=0 UNITS=0
HERE=$(cd "$(dirname "$0")" && pwd)
SIM_RAM="" SIM_DISK="" SIM_ARCH="" SIM_V6=""
STATE_DIR="${APPARATUS_NODE_DIR:-$HOME/.apparatus-node}"

die() { echo "node-up: $*" >&2; exit 1; }
while [ $# -gt 0 ]; do
  case "$1" in
    --nodes-md) NODES_MD="$2"; shift 2 ;;
    --apply) APPLY=1; shift ;;
    --role) ROLE="$2"; shift 2 ;;
    --bootstrap) BOOTSTRAP=1; shift ;;
    --units) UNITS=1; shift ;;
    --node-id) NODE_ID="$2"; shift 2 ;;
    --simulate-ram) SIM_RAM="$2"; shift 2 ;;
    --simulate-disk) SIM_DISK="$2"; shift 2 ;;
    --simulate-arch) SIM_ARCH="$2"; shift 2 ;;
    --simulate-ipv6) SIM_V6="$2"; shift 2 ;;
    -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
    *) die "unknown argument $1" ;;
  esac
done
if [ -n "$SIM_RAM$SIM_DISK$SIM_ARCH$SIM_V6" ] && [ "$APPLY" = 1 ]; then
  die "--simulate-* is for dry runs only; refusing --apply on simulated numbers"
fi
[ "$(uname -s)" = Linux ] || die "Linux only (got $(uname -s))"
case "$ROLE" in vps|anker|pi|laptop|generic) ;; *) die "--role: vps | anker | pi | laptop | generic" ;; esac
[ -n "$NODE_ID" ] || NODE_ID="$ROLE-$(hostname -s 2>/dev/null || echo node)"

# 0. Floors from NODES.md, or stop.
[ -n "$NODES_MD" ] || for c in ./NODES.md ../NAP-corpus/docs/ic/NODES.md "$HOME/NAP-corpus/docs/ic/NODES.md"; do
  [ -f "$c" ] && { NODES_MD="$c"; break; }
done
[ -n "$NODES_MD" ] && [ -f "$NODES_MD" ] || die "NODES.md not found (pass --nodes-md PATH); no floors, no install"

to_bytes() { # "2 GiB" | "100 GB" | "256 MiB" -> bytes
  awk -v n="$1" -v u="$2" 'BEGIN { m = (u=="GiB") ? 1073741824 : (u=="MiB") ? 1048576 : (u=="GB") ? 1e9 : (u=="MB") ? 1e6 : -1;
    if (m < 0) exit 1; printf "%.0f\n", n * m }'
}
floor() { # tier column(4=RAM 5=disk) -> bytes
  local cell
  cell=$(awk -F'|' -v t="**$1**" -v c="$2" '$2 ~ /\*\*/ { g=$2; gsub(/ /,"",g); if (g==t) { x=$c; gsub(/\*|^ +| +$/,"",x); print x; exit } }' "$NODES_MD")
  [ -n "$cell" ] || die "NODES.md has no $1 floor row"
  # shellcheck disable=SC2086
  to_bytes $cell || die "cannot read floor '$cell' for $1"
}
FULL_RAM=$(floor full 4); FULL_DISK=$(floor full 5)
CORE_RAM=$(floor core 4); CORE_DISK=$(floor core 5)
LIGHT_RAM=$(floor light 4); LIGHT_DISK=$(floor light 5)

# 1. Measure.
RAM=${SIM_RAM:-$(awk '/^MemTotal:/ { print $2 * 1024 }' /proc/meminfo)}
DISK=${SIM_DISK:-$(df -B1 --output=avail "$HOME" | tail -1 | tr -d ' ')}
ARCH=${SIM_ARCH:-$(uname -m)}
if [ -n "$SIM_V6" ]; then V6=$SIM_V6
elif command -v ip >/dev/null && ip -6 route show default 2>/dev/null | grep -q .; then V6=1
else V6=0; fi
case "$ARCH" in x86_64|amd64) X86=1 ;; aarch64|arm64) X86=0 ;; *) die "arch $ARCH: neither x86-64 nor arm64; not supported (a 32-bit Pi OS needs the 64-bit image)" ;; esac

if   [ "$RAM" -ge "$FULL_RAM" ] && [ "$DISK" -ge "$FULL_DISK" ] && [ "$X86" = 1 ]; then TIER=full
elif [ "$RAM" -ge "$CORE_RAM" ] && [ "$DISK" -ge "$CORE_DISK" ]; then TIER=core
elif [ "$RAM" -ge "$LIGHT_RAM" ] && [ "$DISK" -ge "$LIGHT_DISK" ]; then TIER=light
else TIER=refused; fi

echo "mode      $([ "$APPLY" = 1 ] && echo apply || echo dry-run)$([ -n "$SIM_RAM$SIM_DISK$SIM_ARCH$SIM_V6" ] && echo ' (simulated shape)')"
echo "role      $ROLE (node id $NODE_ID)"
echo "floors    $NODES_MD: full ${FULL_RAM}/${FULL_DISK} core ${CORE_RAM}/${CORE_DISK} light ${LIGHT_RAM}/${LIGHT_DISK} (RAM/disk bytes)"
echo "measured  ram ${RAM} B, disk_free ${DISK} B, arch ${ARCH}, ipv6 $([ "$V6" = 1 ] && echo yes || echo no)"
echo "tier      $TIER"

# 2. Refuse under the floor. A full install is never offered below the full floor.
[ "$TIER" != refused ] || die "under the light floor: nothing to install"
if [ "$TIER" != full ]; then
  why=""; [ "$X86" = 1 ] || why="arm64 has no stock replica binary"
  [ "$RAM" -ge "$FULL_RAM" ] || why="${why:+$why; }RAM under ${FULL_RAM}"
  [ "$DISK" -ge "$FULL_DISK" ] || why="${why:+$why; }disk under ${FULL_DISK}"
  echo "full      refused: $why. Offered: $TIER"
fi

# 3. What this tier installs. Light and core never download the replica.
case "$TIER" in
  full)  echo "install   replica: NOT by this script; full nodes use the anker path (scripts/ic-up.sh, SHA256SUMS-verified stock binary)" ;;
  core)  echo "install   apparatus CLI + verifier only; replica: not downloaded (core holds roots only)" ;;
  light) echo "install   apparatus CLI + cache only; replica: not downloaded" ;;
esac
if [ "$V6" = 1 ]; then echo "orchestr  not tried by this script (NAP #24: only with IPv6, and only on the anker path)"
else echo "orchestr  skipped: no IPv6 default route (NAP #24)"; fi
echo "network   no public port is opened; replica stays localhost (NAP #27)"
if [ "$ROLE" = anker ] && [ "$TIER" != full ]; then
  echo "anker     WARNING: the anker runs the replica, which needs the full tier; this machine is $TIER"
fi
BOOT_ARGS=(--tier "$TIER"); [ "$APPLY" = 1 ] && BOOT_ARGS+=(--apply)
if [ "$BOOTSTRAP" = 1 ]; then "$HERE/node-bootstrap.sh" "${BOOT_ARGS[@]}"
else echo "toolchain not touched (add --bootstrap; plan: scripts/node-bootstrap.sh --tier $TIER)"; fi
if [ "$UNITS" = 1 ]; then "$HERE/node-units.sh" "${BOOT_ARGS[@]}" --role "$ROLE"
else echo "units     not written (add --units; plan: scripts/node-units.sh --tier $TIER --role $ROLE)"; fi

if [ "$APPLY" = 0 ]; then
  echo "keys      would generate $STATE_DIR/node.key (0600, Ed25519 seed, never printed) and print the principal"
  echo "tailscale $([ -n "${TS_AUTHKEY:-}" ] && echo 'TS_AUTHKEY set: would join' || echo 'TS_AUTHKEY not set: would stop before join')"
  echo "register  would write $STATE_DIR/node.json (id $NODE_ID, tier $TIER, specs, addresses) for the anker to pull"
  echo "dry run: nothing changed"
  exit 0
fi

# 4. Keys. Seed is 32 raw bytes, 0600, never printed. Principal = self-authenticating (Ed25519 DER).
command -v openssl >/dev/null || die "openssl missing"
command -v python3 >/dev/null || die "python3 missing"
umask 077; mkdir -p "$STATE_DIR"
if [ ! -f "$STATE_DIR/node.key" ]; then head -c 32 /dev/urandom > "$STATE_DIR/node.key"; fi
chmod 600 "$STATE_DIR/node.key"
PUB=$( { printf '\x30\x2e\x02\x01\x00\x30\x05\x06\x03\x2b\x65\x70\x04\x22\x04\x20'; cat "$STATE_DIR/node.key"; } \
  | openssl pkey -inform DER -pubout -outform DER | od -An -tx1 | tr -d ' \n')
PRINCIPAL=$(python3 - "$PUB" <<'PY'
import base64, hashlib, sys, zlib
b = hashlib.sha224(bytes.fromhex(sys.argv[1])).digest() + b"\x02"
s = base64.b32encode(zlib.crc32(b).to_bytes(4, "big") + b).decode().lower().rstrip("=")
print("-".join(s[i:i + 5] for i in range(0, len(s), 5)))
PY
)
echo "principal $PRINCIPAL"

# 5. Tailscale. Only with a key already in the environment. Never invented.
if [ -z "${TS_AUTHKEY:-}" ]; then
  echo "tailscale TS_AUTHKEY not set: stopping before join"
else
  command -v tailscale >/dev/null || die "tailscale not installed; install it from your distribution, then rerun"
  sudo tailscale up --authkey "$TS_AUTHKEY" --hostname "$NODE_ID" --ssh
  echo "tailscale joined as $NODE_ID"
fi

# 6. Register locally. The anker pulls this file over Tailscale SSH; nothing is pushed.
ADDRS=$( (ip -o addr show scope global 2>/dev/null || true) | awk '{print $4}' | paste -sd, -)
cat > "$STATE_DIR/node.json" <<JSON
{"id":"$NODE_ID","role":"$ROLE","tier":"$TIER","principal":"$PRINCIPAL","ram_bytes":$RAM,"disk_free_bytes":$DISK,"arch":"$ARCH","ipv6":$([ "$V6" = 1 ] && echo true || echo false),"addresses":"$ADDRS","measured_at":"$(date -u +%FT%TZ)"}
JSON
chmod 600 "$STATE_DIR/node.json"
echo "register  $STATE_DIR/node.json"
