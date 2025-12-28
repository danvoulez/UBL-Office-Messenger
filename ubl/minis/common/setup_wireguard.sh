#!/usr/bin/env bash
set -euo pipefail
if ! command -v brew >/dev/null ; then echo "Homebrew requerido"; exit 1; fi
brew list wireguard-tools >/dev/null 2>&1 || brew install wireguard-tools wireguard-go
sudo mkdir -p /usr/local/etc/wireguard
sudo mv /tmp/wg0.conf /usr/local/etc/wireguard/wg0.conf
sudo wg-quick down wg0 >/dev/null 2>&1 || true
sudo wg-quick up wg0
# launchd plist
cat <<'PL' | sudo tee /Library/LaunchDaemons/com.ubl.wg0.plist >/dev/null
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.ubl.wg0</string>
  <key>ProgramArguments</key>
  <array><string>/usr/local/bin/wg-quick</string><string>up</string><string>wg0</string></array>
  <key>RunAtLoad</key><true/>
</dict></plist>
PL
sudo launchctl load -w /Library/LaunchDaemons/com.ubl.wg0.plist
echo "✅ WireGuard configurado"
