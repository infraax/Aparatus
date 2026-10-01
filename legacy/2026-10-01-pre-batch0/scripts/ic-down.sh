#!/usr/bin/env bash
# Stop the replica started by ic-up.sh (SIGTERM, wait until gone). State persists in $IC_DIR/run.
set -euo pipefail
source "$(dirname "$0")/common.sh"
PIDF="$IC_DIR/run/replica.pid"
if [ ! -f "$PIDF" ] || ! kill -0 "$(cat "$PIDF")" 2>/dev/null; then
  echo "replica not running"; rm -f "$PIDF"; exit 0
fi
PID=$(cat "$PIDF"); kill -TERM "$PID"
while kill -0 "$PID" 2>/dev/null; do sleep 0.5; done
rm -f "$PIDF"; echo "replica stopped (pid $PID)"
