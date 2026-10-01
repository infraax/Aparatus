#!/usr/bin/env bash
# Start one stock IC replica (N=1, SubnetType::System, f=0) as in NAP-corpus docs/ic/LOCAL-RUN.md.
# Stock release binaries only, verified against SHA256SUMS. Never builds or patches dfinity/ic.
set -euo pipefail
source "$(dirname "$0")/common.sh"
mkdir -p "$IC_DIR/bin"
cd "$IC_DIR/bin"

# True when every .gz matches SHA256SUMS and every binary is exactly its .gz unpacked.
verified() {
  [ -f SHA256SUMS ] && echo "$IC_SUMS_SHA256  SHA256SUMS" | sha256sum -c --quiet - > /dev/null 2>&1 || return 1
  for f in $IC_BINS; do grep " $f.gz\$" SHA256SUMS; done | sha256sum -c --quiet - > /dev/null 2>&1 || return 1
  for f in $IC_BINS; do [ -x "$f" ] && gunzip -c "$f.gz" | cmp -s - "$f" || return 1; done
}

if verified; then
  echo "binaries already verified against SHA256SUMS; no download"
else
  echo "downloading $IC_COMMIT binaries"
  curl -fsSO "$IC_BASE/SHA256SUMS"
  echo "$IC_SUMS_SHA256  SHA256SUMS" | sha256sum -c - || { echo "SHA256SUMS does not match the pinned digest; refusing" >&2; exit 1; }
  for f in $IC_BINS; do curl -fsSO "$IC_BASE/$f.gz"; done
  for f in $IC_BINS; do grep " $f.gz\$" SHA256SUMS; done | sha256sum -c -
  for f in $IC_BINS; do gunzip -kf "$f.gz"; chmod +x "$f"; done
  verified || { echo "verification failed after download; refusing" >&2; exit 1; }
  echo "binaries verified"
fi
cd "$IC_DIR"

# Identities (docs: NAP-corpus docs/ic/canisters/IDENTITIES.md). Raw Ed25519 seeds, mode 0600, git-ignored.
#   dev.key   — installs and upgrades canisters (controller), creates canisters
#   agent.key — may create nothing it controls; only calls methods it is allowed to (e.g. notary record)
cargo build --locked -q --manifest-path "$ROOT/Cargo.toml" -p apparatus-cli
APP="$ROOT/target/debug/apparatus"
DEV_PRINCIPAL=$("$APP" rws ic key "$IC_DIR/dev.key" | cut -d' ' -f1)
AGENT_PRINCIPAL=$("$APP" rws ic key "$IC_DIR/agent.key" | cut -d' ' -f1)

# Bootstrap once: one node, one subnet (index 0 = System/root). The provisional whitelist holds exactly the
# two principals above: an unknown principal cannot create a canister on this replica.
WANT="{\"provisional_whitelist\": [\"$DEV_PRINCIPAL\", \"$AGENT_PRINCIPAL\"]}"
if [ -d prep/ic_registry_local_store ] && [ "$(cat prep/whitelist.json 2>/dev/null)" != "$WANT" ]; then
  echo "$IC_DIR/prep was bootstrapped with a different provisional whitelist than dev.key + agent.key." >&2
  echo "Re-bootstrap: scripts/ic-down.sh && rm -rf $IC_DIR/prep $IC_DIR/run $IC_DIR/replica.json5" >&2
  exit 1
fi
if [ ! -d prep/ic_registry_local_store ]; then
  mkdir -p prep
  echo "$WANT" > prep/whitelist.json
  ./bin/ic-prep --working-dir "$IC_DIR/prep" --replica-version $IC_COMMIT --allow-empty-update-image \
    --provisional-whitelist "$IC_DIR/prep/whitelist.json" \
    --node 'idx:0,subnet_idx:0,xnet_api:"127.0.0.1:2497",public_api:"127.0.0.1:8080"'
fi
echo "identities: dev $DEV_PRINCIPAL ($IC_DIR/dev.key), agent $AGENT_PRINCIPAL ($IC_DIR/agent.key)"

# Config once: the stock sample with only the fields that must change.
if [ ! -f replica.json5 ]; then
  mkdir -p run/state run/pool run/backup
  [ -d run/crypto ] || { cp -r prep/node-0/crypto run/crypto && chmod 700 run/crypto; }
  ./bin/replica --replica-version $IC_COMMIT --guestos-version $IC_COMMIT --print-sample-config > sample.json5
  sed -e "s|/var/lib/ic/data/ic_registry_local_store/|$IC_DIR/prep/ic_registry_local_store/|" \
      -e "s|\"/tmp/ic_state\"|\"$IC_DIR/run/state\"|" \
      -e "s|\"/tmp/ic_consensus_pool\"|\"$IC_DIR/run/pool\"|" \
      -e "s|\"/tmp/ic_backup/\"|\"$IC_DIR/run/backup/\"|" \
      -e "s|\"/tmp/ic_crypto\"|\"$IC_DIR/run/crypto\"|" \
      -e 's|csp_vault_type: { unix_socket: {.*} },|csp_vault_type: "in_replica",|' \
      -e 's|listening_port: 3000|listening_port: 2497|' \
      -e "s|\"/tmp/bitcoin_uds\"|\"$IC_DIR/run/bitcoin.sock\"|" \
      -e "s|\"/run/ic-node/https-outcalls-adapter/socket\"|\"$IC_DIR/run/https-outcalls.sock\"|" \
      sample.json5 > replica.json5
fi

PIDF="$IC_DIR/run/replica.pid"
if [ -f "$PIDF" ] && kill -0 "$(cat "$PIDF")" 2>/dev/null; then
  echo "replica already running (pid $(cat "$PIDF")): $IC_STATUS_URL"; exit 0
fi
if curl -sf -o /dev/null "$IC_STATUS_URL"; then
  echo "something else already answers on $IC_STATUS_URL; refusing to start a second replica" >&2; exit 1
fi
(cd "$IC_DIR/bin" && exec nohup ./replica --replica-version $IC_COMMIT --guestos-version $IC_COMMIT \
  --config-file "$IC_DIR/replica.json5" >> "$IC_DIR/run/replica.log" 2>&1) &
echo $! > "$PIDF"
for _ in $(seq 1 60); do
  curl -sf -o /dev/null "$IC_STATUS_URL" && { echo "replica up (pid $(cat "$PIDF")): $IC_STATUS_URL"; exit 0; }
  kill -0 "$(cat "$PIDF")" 2>/dev/null || { echo "replica exited; see $IC_DIR/run/replica.log" >&2; rm -f "$PIDF"; exit 1; }
  sleep 1
done
echo "no status after 60 s; see $IC_DIR/run/replica.log" >&2; exit 1
