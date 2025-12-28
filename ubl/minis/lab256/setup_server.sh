#!/usr/bin/env bash
set -euo pipefail
mkdir -p /opt/ubl/etc /opt/ubl/logs /opt/ubl/bin
# Se o bin já tiver sido copiado por push_bins.sh, só sobe PM2
pm2 start /opt/ubl/etc/ecosystem.256.config.js --update-env || true
pm2 save
echo "✅ ubl-server sob PM2"
