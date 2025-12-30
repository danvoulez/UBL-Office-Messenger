#!/usr/bin/env bash
set -euo pipefail

# Headless bootstrap for LAB 8GB (local) + LAB 256 / LAB 512 (via SSH)
# Requires: macOS, admin user (dan), and your public key in ~/.ssh/id_ed25519.pub

# ---- Config (set via env before running) ----
: "${LAB256_HOST:?Set LAB256_HOST (e.g., 192.168.1.26)}"
: "${LAB512_HOST:?Set LAB512_HOST (e.g., 192.168.1.51)}"
: "${SSH_USER:=dan}"
: "${AUTHORIZED_KEY:?Set AUTHORIZED_KEY with your public key content}"
: "${SSID_LAB:=lab}"
: "${PASS_LAB:?Set PASS_LAB (Wi-Fi password)}"
: "${SSID_SANTO:=santoandre}"
: "${PASS_SANTO:?Set PASS_SANTO (Wi-Fi password)}"

THIS_HOSTNAME="$(hostname -s || true)"

# ---- Helpers ----
bold() { printf "\033[1m%s\033[0m\n" "$*"; }
run_local() { echo "+ $*"; eval "$@"; }
run_ssh()   { local host="$1"; shift; echo "+ [$host] $*"; ssh -o BatchMode=yes -o PubkeyAuthentication=yes "${SSH_USER}@${host}" "$@"; }
scp_put()   { local src="$1" host="$2" dst="$3"; echo "+ [scp->$host] $src -> $dst"; scp -o BatchMode=yes -o PubkeyAuthentication=yes "$src" "${SSH_USER}@${host}:$dst"; }

wifi_dev() {
  # best-effort detect Wi-Fi device (en0/en1)
  networksetup -listallhardwareports | awk '/Hardware Port: Wi-Fi/{getline; print $2; exit}'
}

configure_node() {
  local TARGET="$1" NEW_HOSTNAME="$2" CONNECT_WIFI="$3"

  run_ssh "$TARGET" "sudo scutil --set HostName ${NEW_HOSTNAME} && sudo scutil --set LocalHostName ${NEW_HOSTNAME} && sudo scutil --set ComputerName '${NEW_HOSTNAME^^}' || true"
  run_ssh "$TARGET" "sudo systemsetup -setcomputersleep Never || true"
  run_ssh "$TARGET" "sudo pmset -a sleep 0 disksleep 0 displaysleep 0 powernap 0 || true"
  run_ssh "$TARGET" "sudo systemsetup -setremotelogin on || true"

  # sshd_config hardening (keep a backup)
  run_ssh "$TARGET" "sudo sed -i.bak -e 's/^#\?PasswordAuthentication .*/PasswordAuthentication no/' -e 's/^#\?PubkeyAuthentication .*/PubkeyAuthentication yes/' /etc/ssh/sshd_config || true"
  run_ssh "$TARGET" "sudo launchctl kickstart -k system/com.openssh.sshd || true"

  # authorized_keys
  run_ssh "$TARGET" "mkdir -p ~/.ssh && chmod 700 ~/.ssh && printf '%s\n' '${AUTHORIZED_KEY}' >> ~/.ssh/authorized_keys && sort -u ~/.ssh/authorized_keys -o ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"

  # Homebrew and tools
  run_ssh "$TARGET" "which brew >/dev/null 2>&1 || /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
  run_ssh "$TARGET" "brew install -q jq coreutils bash git htop iproute2mac || true"

  if [[ "$CONNECT_WIFI" == "yes" ]]; then
    run_ssh "$TARGET" "DEV=\$(networksetup -listallhardwareports | awk '/Hardware Port: Wi-Fi/{getline; print \$2; exit}'); \
      if [[ -n \"\$DEV\" ]]; then \
        networksetup -setairportpower \"\$DEV\" on || true; \
        networksetup -setairportnetwork \"\$DEV\" \"${SSID_LAB}\" \"${PASS_LAB}\" || true; \
        networksetup -addpreferredwirelessnetworkatindex \"\$DEV\" \"${SSID_SANTO}\" 1 WPA2 \"${PASS_SANTO}\" || true; \
      fi"
  fi

  # sanity
  run_ssh "$TARGET" "hostname; pmset -g | egrep -i 'sleep|powernap' || true; ipconfig getifaddr en0 || ipconfig getifaddr en1 || true; ping -c 2 1.1.1.1 || true"
}

bold "==> Local (LAB 8GB): prepare"
DEV="$(wifi_dev || true)"
if [[ -n "${DEV}" ]]; then
  run_local "networksetup -setairportpower ${DEV} on || true"
fi
run_local "sudo systemsetup -setremotelogin on || true"
run_local "mkdir -p ~/.ssh && chmod 700 ~/.ssh"
run_local "grep -q '${AUTHORIZED_KEY}' ~/.ssh/authorized_keys 2>/dev/null || (printf '%s\n' '${AUTHORIZED_KEY}' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys)"

case "${THIS_HOSTNAME}" in
  lab8gb|LAB8GB) run_local "sudo scutil --set HostName lab8gb && sudo scutil --set LocalHostName lab8gb && sudo scutil --set ComputerName 'LAB 8GB' || true" ;;
  *) : ;; # não força renome no local se já estiver ok
esac
run_local "sudo systemsetup -setcomputersleep Never || true"
run_local "sudo pmset -a sleep 0 disksleep 0 displaysleep 0 powernap 0 || true"

bold "==> Remote: LAB 256"
configure_node "$LAB256_HOST" "lab256" "yes"

bold "==> Remote: LAB 512"
configure_node "$LAB512_HOST" "lab512" "yes"

bold "==> DONE. Teste SSH sem senha:"
echo "ssh ${SSH_USER}@${LAB256_HOST} 'echo ok-256'"
echo "ssh ${SSH_USER}@${LAB512_HOST} 'echo ok-512'"
