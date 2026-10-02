#!/usr/bin/env bash
# Toolchain for a node tier, on Debian/Ubuntu (apt), Fedora/RHEL/Oracle Linux (dnf) or Arch (pacman).
#
#   scripts/node-bootstrap.sh --tier light|core|full [--apply] [--with-tailscale]
#
# Default is a DRY RUN: prints the exact commands and changes nothing. --apply runs them (sudo for
# system packages; rustup and mops go to the user's home). Nothing here touches keys.
#   light  git curl ca-certificates openssl python3            (CLI from a release, cache only)
#   core   + build tools, pkg-config, OpenSSL headers, rustup   (builds apparatus + verifier)
#   full   + Node.js and npm (mops, moc) and the replica's      (anker path: scripts/ic-up.sh)
#            runtime libraries
# --with-tailscale adds Tailscale from its own package repository (tailscale.com/install.sh).
set -euo pipefail
TIER="" APPLY=0 TS=0
die() { echo "node-bootstrap: $*" >&2; exit 1; }
while [ $# -gt 0 ]; do
  case "$1" in
    --tier) TIER="$2"; shift 2 ;;
    --apply) APPLY=1; shift ;;
    --with-tailscale) TS=1; shift ;;
    -h|--help) sed -n '2,13p' "$0"; exit 0 ;;
    *) die "unknown argument $1" ;;
  esac
done
case "$TIER" in light|core|full) ;; *) die "--tier light|core|full" ;; esac
[ "$(uname -s)" = Linux ] || die "Linux only"

if command -v apt-get > /dev/null; then PM=apt
elif command -v dnf > /dev/null; then PM=dnf
elif command -v pacman > /dev/null; then PM=pacman
else die "no apt-get, dnf or pacman: install by hand (see the tier list in this script)"; fi

pkgs() { # tier -> package names for $PM
  local light core full
  case "$PM" in
    apt)    light="git curl ca-certificates openssl python3"; core="build-essential pkg-config libssl-dev"; full="nodejs npm libunwind8 zstd" ;;
    dnf)    light="git curl ca-certificates openssl python3"; core="gcc gcc-c++ make pkgconf-pkg-config openssl-devel"; full="nodejs npm libunwind zstd" ;;
    pacman) light="git curl ca-certificates openssl python"; core="base-devel pkgconf openssl"; full="nodejs npm libunwind zstd" ;;
  esac
  case "$1" in light) echo "$light" ;; core) echo "$light $core" ;; full) echo "$light $core $full" ;; esac
}
CMDS=()
case "$PM" in
  apt)    CMDS+=("sudo apt-get update" "sudo apt-get install -y --no-install-recommends $(pkgs "$TIER")") ;;
  dnf)    CMDS+=("sudo dnf install -y $(pkgs "$TIER")") ;;
  pacman) CMDS+=("sudo pacman -S --needed --noconfirm $(pkgs "$TIER")") ;;
esac
if [ "$TIER" != light ] && ! command -v cargo > /dev/null; then
  CMDS+=("curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable")
fi
[ "$TIER" = full ] && CMDS+=("echo 'mops and moc come from npm ci in NAP-corpus docs/ic/canisters (package-lock.json)'")
if [ "$TS" = 1 ] && ! command -v tailscale > /dev/null; then
  CMDS+=("curl -fsSL https://tailscale.com/install.sh | sh")
fi

echo "bootstrap $TIER via $PM ($([ "$APPLY" = 1 ] && echo apply || echo dry-run))"
for c in "${CMDS[@]}"; do echo "  \$ $c"; done
[ "$APPLY" = 1 ] || { echo "bootstrap dry run: nothing installed"; exit 0; }
for c in "${CMDS[@]}"; do bash -c "$c"; done
echo "bootstrap done: $(command -v cargo > /dev/null && cargo --version || echo 'no cargo (light tier)')"
