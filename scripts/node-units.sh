#!/usr/bin/env bash
# systemd --user units for a node, so its services survive logout and reboot (with lingering).
#
#   scripts/node-units.sh --tier light|core|full --role ROLE [--apply]
#
# Default is a DRY RUN: prints the unit files and changes nothing. --apply writes them to
# ~/.config/systemd/user/, reloads, and enables them (it does not start them).
#   core, full   apparatusd.service   the single writer for this repo (socket 0600)
#   full         ic-replica.service   the local replica via scripts/ic-up.sh / ic-down.sh
# Light nodes run no service. Units bind nothing public: the replica stays on 127.0.0.1.
set -euo pipefail
TIER="" ROLE=generic APPLY=0
die() { echo "node-units: $*" >&2; exit 1; }
while [ $# -gt 0 ]; do
  case "$1" in
    --tier) TIER="$2"; shift 2 ;;
    --role) ROLE="$2"; shift 2 ;;
    --apply) APPLY=1; shift ;;
    -h|--help) sed -n '2,11p' "$0"; exit 0 ;;
    *) die "unknown argument $1" ;;
  esac
done
case "$TIER" in light|core|full) ;; *) die "--tier light|core|full" ;; esac
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DIR="$HOME/.config/systemd/user"
[ "$TIER" = light ] && { echo "units     light tier: no services"; exit 0; }

unit_apparatusd() {
  cat <<UNIT
[Unit]
Description=apparatusd: the single writer for $ROOT ($ROLE)

[Service]
Type=simple
Environment=PATH=%h/.cargo/bin:/usr/local/bin:/usr/bin:/bin
ExecStartPre=/usr/bin/env cargo build --manifest-path $ROOT/Cargo.toml --locked -q -p apparatusd
ExecStart=$ROOT/target/debug/apparatusd --project $ROOT
Restart=on-failure
UMask=0077

[Install]
WantedBy=default.target
UNIT
}
unit_replica() {
  cat <<UNIT
[Unit]
Description=Local IC replica for $ROOT (stock binaries, verified by scripts/ic-up.sh)

[Service]
Type=forking
Environment=PATH=%h/.cargo/bin:/usr/local/bin:/usr/bin:/bin
ExecStart=$ROOT/scripts/ic-up.sh
ExecStop=$ROOT/scripts/ic-down.sh
TimeoutStartSec=900
Restart=no

[Install]
WantedBy=default.target
UNIT
}
UNITS=(apparatusd); [ "$TIER" = full ] && UNITS+=(ic-replica)
echo "units     $TIER ($ROLE): ${UNITS[*]} -> $DIR ($([ "$APPLY" = 1 ] && echo apply || echo dry-run))"
for u in "${UNITS[@]}"; do
  if [ "$APPLY" = 0 ]; then
    echo "--- $u.service"
    case "$u" in apparatusd) unit_apparatusd ;; ic-replica) unit_replica ;; esac
    continue
  fi
  mkdir -p "$DIR"
  case "$u" in apparatusd) unit_apparatusd ;; ic-replica) unit_replica ;; esac > "$DIR/$u.service"
done
[ "$APPLY" = 1 ] || { echo "units dry run: nothing written"; exit 0; }
systemctl --user daemon-reload
for u in "${UNITS[@]}"; do systemctl --user enable "$u.service"; done
echo "units     enabled (not started). For start at boot without a login: sudo loginctl enable-linger $USER"
