#!/usr/bin/env bash
set -euo pipefail
mkdir -p /opt/ubl/etc /opt/ubl/logs /opt/ubl/bin
pm2 start /opt/ubl/etc/ecosystem.512.config.js --update-env || true
pm2 save
echo "✅ ubl-runner sob PM2"
