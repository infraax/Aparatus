#!/usr/bin/env bash
# Kept for the commands in docs/homelab/VPS.md: the Oracle VPS is node-up.sh with --role vps.
exec "$(dirname "$0")/node-up.sh" --role vps "$@"
