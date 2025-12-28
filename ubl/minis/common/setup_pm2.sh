#!/usr/bin/env bash
set -euo pipefail
if ! command -v brew >/dev/null ; then echo "Homebrew requerido"; exit 1; fi
brew list node >/dev/null 2>&1 || brew install node
npm i -g pm2
mkdir -p /opt/ubl/bin /opt/ubl/etc /opt/ubl/logs
# launchd plist para resurrect no boot
cat <<'PL' | sudo tee /Library/LaunchDaemons/com.ubl.pm2.plist >/dev/null
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.ubl.pm2</string>
  <key>ProgramArguments</key>
  <array><string>/usr/local/bin/pm2</string><string>resurrect</string></array>
  <key>RunAtLoad</key><true/>
</dict></plist>
PL
sudo launchctl load -w /Library/LaunchDaemons/com.ubl.pm2.plist
echo "✅ PM2 pronto"
