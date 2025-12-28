#!/usr/bin/env bash
set -euo pipefail
ANCHOR="/etc/pf.anchors/ubl"
sudo bash -c 'cat > '"$ANCHOR" <<'PF'
# allow WireGuard subnet only
table <ubl_wg> persist { 10.8.0.0/24 }
set block-policy drop
block all
pass quick on utun0 from <ubl_wg> to any keep state
pass quick on lo0 all
PF
sudo bash -c 'cat > /etc/pf.conf <<PFC
set skip on lo0
anchor "ubl"
load anchor "ubl" from "$ANCHOR"
PFC'
sudo pfctl -f /etc/pf.conf
sudo pfctl -e || true
echo "✅ pf (firewall) aplicado"
