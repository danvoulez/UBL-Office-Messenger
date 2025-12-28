#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
[ -f .env ] && source .env || true

inv="../controller/inventory.json"
WG_USER="${WG_USER:-wguser}"
SSH_KEY="${SSH_KEY:-~/.ssh/ubl}"

function _ssh() { local host="$1"; shift; ssh -i "${SSH_KEY}" -o StrictHostKeyChecking=no -o ConnectTimeout=8 "${WG_USER}@${host}" "$@"; }
function _scp() { scp -i "${SSH_KEY}" -o StrictHostKeyChecking=no "$1" "$2"; }

host_ip_bootstrap() { python3 - "$1" <<PY
import json,os,sys
inv=json.load(open("${inv}"))
boot=inv[sys.argv[1]]["bootstrap_ip"]
print(os.path.expandvars(boot))
PY
}

host_ip_wg() { python3 - "$1" <<PY
import json,sys
inv=json.load(open("${inv}"))
print(inv[sys.argv[1]]["wg_ip"])
PY
}

push_wg() { local key="$1" ; local host="$2"
  _scp "../wg/wg0.${key}.conf" "${WG_USER}@${host}:/tmp/wg0.conf"
  _ssh "${host}" 'bash -s' < ../minis/common/setup_wireguard.sh
}

post_basics() { local host="$1"
  _ssh "${host}" 'bash -s' < ../minis/common/setup_pmset.sh
  _ssh "${host}" 'bash -s' < ../minis/common/setup_pf.sh
  _ssh "${host}" 'bash -s' < ../minis/common/setup_pm2.sh
}

install_server() { local host="$1"
  _ssh "${host}" 'bash -s' < ../minis/lab256/setup_postgres.sh
  _ssh "${host}" 'bash -s' < ../minis/lab256/setup_minio.sh
  bash controller/push_bins.sh lab256
}

install_runner() { local host="$1"
  bash controller/push_bins.sh lab512
}

wifi_join() { local host="$1"
  _ssh "${host}" "sudo /System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport -z || true"
  _ssh "${host}" "networksetup -setairportpower en0 on || true"
  _ssh "${host}" "networksetup -setairportnetwork en0 '${WIFI_SSID}' '${WIFI_PASS}'"
  echo "✅ Wi-Fi solicitado no host ${host}"
}

health() {
  for h in "$(host_ip_wg lab256)" "$(host_ip_wg lab512)"; do
    _ssh "${h}" "pm2 status || true; curl -s http://localhost:8080/health || curl -s http://localhost:9090/health || true"
  done
}

smoke() { bash ../scripts/e2e_smoke.sh || true; }

case "${1:-}" in
  a1-bootstrap)
    key="${2:?lab256|lab512}"
    boot="$(host_ip_bootstrap ${key})"
    [ -z "${boot}" ] && { echo "Defina ${key}.bootstrap_ip"; exit 1; }
    push_wg "${key}" "${boot}"
    post_basics "${boot}"
    if [ "${key}" = "lab256" ]; then install_server "${boot}"; else install_runner "${boot}"; fi
    ;;
  b1-join)
    key="${2:?lab256|lab512}"
    boot="$(host_ip_bootstrap ${key})"
    [ -z "${boot}" ] && { echo "Defina ${key}.bootstrap_ip"; exit 1; }
    wifi_join "${boot}"
    push_wg "${key}" "${boot}"
    post_basics "${boot}"
    if [ "${key}" = "lab256" ]; then install_server "${boot}"; else install_runner "${boot}"; fi
    ;;
  post-wg)
    key="${2:?lab256|lab512}"
    host="$(host_ip_wg ${key})"
    post_basics "${host}"
    if [ "${key}" = "lab256" ]; then install_server "${host}"; else install_runner "${host}"; fi
    ;;
  health) health ;;
  smoke) smoke ;;
  *)
    echo "Uso:"
    echo "  a1-bootstrap lab256|lab512"
    echo "  b1-join lab256|lab512"
    echo "  post-wg lab256|lab512"
    echo "  health | smoke"
    ;;
esac
