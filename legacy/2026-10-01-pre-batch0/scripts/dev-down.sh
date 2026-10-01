#!/usr/bin/env bash
# Stop the apparatusd started by dev-up.sh (SIGTERM: finishes queued requests, removes the socket).
set -euo pipefail
source "$(dirname "$0")/common.sh"
PIDF="$APP_DIR/apparatusd.pid"
if [ ! -f "$PIDF" ] || ! kill -0 "$(cat "$PIDF")" 2>/dev/null; then
  echo "apparatusd not running"; rm -f "$PIDF"; exit 0
fi
PID=$(cat "$PIDF"); kill -TERM "$PID"
while kill -0 "$PID" 2>/dev/null; do sleep 0.2; done
rm -f "$PIDF"; echo "apparatusd stopped (pid $PID)"
