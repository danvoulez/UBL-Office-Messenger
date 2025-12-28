#!/usr/bin/env bash
set -euo pipefail
sudo pmset -a sleep 0
sudo pmset -a displaysleep 0
sudo pmset -a disksleep 0
sudo pmset -a powernap 0
sudo systemsetup -setcomputersleep Never 2>/dev/null || true
echo "✅ pmset aplicado (sem hibernação)"
