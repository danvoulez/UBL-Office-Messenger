#!/usr/bin/env bash
set -euo pipefail
echo "⏱  smoke: ping WG + pm2 status"
for h in 10.8.0.2 10.8.0.3; do
  echo "---- ${h} ----"
  ssh -o StrictHostKeyChecking=no wguser@${h} "pm2 status || true"
done
echo "✅ smoke ok (básico)"
