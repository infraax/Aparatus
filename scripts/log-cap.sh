#!/usr/bin/env bash
# Cap run/replica.log (NAP-corpus #48, Design B as a cap; docs/homelab/LOG-POLICY.md §6).
# If the log is at or over the boundary: copy it to a closed segment OUTSIDE the state dir,
# hash the segment, verify the copy byte-for-byte, record {seq, file, bytes, sha256, prev}
# in segments.jsonl, then truncate the live log. Segments are never deleted or rewritten.
# Runs only while the replica is stopped (no lines can land between copy and truncate).
# The segment sha256 is a leaf the anker may later seal into a daily root; this script
# sends nothing anywhere.
#   IC_LOG_CAP_BYTES  boundary, default 67108864 (64 MiB)
#   IC_LOG_SEGMENTS   destination, default $IC_DIR/log-segments
set -euo pipefail
source "$(dirname "$0")/common.sh"
LOG="$IC_DIR/run/replica.log"
CAP="${IC_LOG_CAP_BYTES:-67108864}"
SEG="${IC_LOG_SEGMENTS:-$IC_DIR/log-segments}"
PIDF="$IC_DIR/run/replica.pid"
if [ -f "$PIDF" ] && kill -0 "$(cat "$PIDF")" 2>/dev/null; then
  echo "log-cap: replica is running (pid $(cat "$PIDF")); refusing (stop it first)" >&2; exit 1
fi
[ -f "$LOG" ] || { echo "log-cap: no $LOG"; exit 0; }
BYTES=$(stat -c %s "$LOG")
if [ "$BYTES" -lt "$CAP" ]; then echo "log-cap: $BYTES bytes < cap $CAP; nothing to close"; exit 0; fi
mkdir -p "$SEG"; chmod 700 "$SEG"
MAN="$SEG/segments.jsonl"
SEQ=0; PREV=""
if [ -s "$MAN" ]; then
  SEQ=$(( $(wc -l < "$MAN") ))
  PREV=$(tail -1 "$MAN" | sed -n 's/.*"sha256":"\([0-9a-f]*\)".*/\1/p')
fi
FIRST=$(head -c 15 "$LOG" | tr -c '[:alnum:]' '_')
CLOSED=$(date -u +%Y%m%dT%H%M%SZ)
OUT="$SEG/replica.$(printf %06d "$SEQ").$CLOSED.log"
[ -e "$OUT" ] && { echo "log-cap: $OUT exists; refusing to overwrite" >&2; exit 1; }
cp "$LOG" "$OUT"
SRC=$(sha256sum "$LOG" | cut -d' ' -f1)
DST=$(sha256sum "$OUT" | cut -d' ' -f1)
[ "$SRC" = "$DST" ] && [ "$(stat -c %s "$OUT")" = "$BYTES" ] || { echo "log-cap: copy does not match ($SRC vs $DST); log left untouched" >&2; exit 1; }
chmod 0444 "$OUT"
printf '{"seq":%d,"file":"%s","bytes":%d,"sha256":"%s","prev_sha256":"%s","first_line_prefix":"%s","closed_utc":"%s"}\n' \
  "$SEQ" "$(basename "$OUT")" "$BYTES" "$DST" "$PREV" "$FIRST" "$CLOSED" >> "$MAN"
: > "$LOG"
echo "log-cap: closed segment $SEQ: $OUT ($BYTES bytes, sha256 $DST, prev ${PREV:-none}); replica.log truncated"
