#!/usr/bin/env bash
# Start apparatusd for this repo (project = repo root, .apparatus/). Prints the socket path.
set -euo pipefail
source "$(dirname "$0")/common.sh"
cd "$ROOT"
cargo build --locked -q -p apparatus-cli -p apparatusd
BIN="$ROOT/target/debug"
SOCK="$APP_DIR/apparatusd.sock"
PIDF="$APP_DIR/apparatusd.pid"
if [ -f "$PIDF" ] && kill -0 "$(cat "$PIDF")" 2>/dev/null; then
  echo "apparatusd already running (pid $(cat "$PIDF"))"
  echo "socket: $SOCK"
  exit 0
fi
if [ ! -f "$APP_DIR/project.json" ]; then
  "$BIN/apparatus" --project "$ROOT" rws init --solo > /dev/null
  echo "initialised solo project in $APP_DIR"
fi
nohup "$BIN/apparatusd" --project "$ROOT" >> "$APP_DIR/apparatusd.log" 2>&1 &
echo $! > "$PIDF"
for _ in $(seq 1 50); do
  [ -S "$SOCK" ] && break
  if ! kill -0 "$(cat "$PIDF")" 2>/dev/null; then
    echo "apparatusd exited; last log lines:" >&2; tail -5 "$APP_DIR/apparatusd.log" >&2; rm -f "$PIDF"; exit 1
  fi
  sleep 0.2
done
[ -S "$SOCK" ] || { echo "no socket after 10 s" >&2; exit 1; }
echo "apparatusd pid $(cat "$PIDF"), log $APP_DIR/apparatusd.log"
echo "socket: $SOCK"
