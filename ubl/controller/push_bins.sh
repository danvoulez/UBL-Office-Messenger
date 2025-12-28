#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
[ -f .env ] && source .env || true

inv_json="../controller/inventory.json"
manifest_json="../controller/install.manifest.json"

jq_bin=$(command -v jq || true)
if [ -z "${jq_bin}" ]; then echo "Instale jq (brew install jq)"; exit 1; fi

WG_USER="${WG_USER:-wguser}"
SSH_KEY="${SSH_KEY:-~/.ssh/ubl}"

_subst() { # string with ${var} expansion from .env and defaults
  local s="$1"
  eval "echo \"$s\""
}

_host_ip() {
  local key="$1"
  python3 - "$key" <<PY
import json,os,sys
m=json.load(open("${manifest_json}"))
val=m["hosts"][sys.argv[1]]["wg_ip"]
print(os.path.expandvars(val))
PY
}

_push_file() { # src dsthost dstdir dstname mode(optional)
  local src="$1" ; local host="$2" ; local dst="$3" ; local mode="${4:-}"
  ssh -i "${SSH_KEY}" -o StrictHostKeyChecking=no "${WG_USER}@${host}" "sudo mkdir -p $(dirname "$dst") && sudo chown ${WG_USER} $(dirname "$dst")"
  scp -i "${SSH_KEY}" -o StrictHostKeyChecking=no "$src" "${WG_USER}@${host}:$dst"
  if [ -n "$mode" ]; then
    ssh -i "${SSH_KEY}" -o StrictHostKeyChecking=no "${WG_USER}@${host}" "chmod $mode $dst"
  fi
}

_do_service() { # host, cmd
  local host="$1" ; shift
  ssh -i "${SSH_KEY}" -o StrictHostKeyChecking=no "${WG_USER}@${host}" "$@"
}

publish_host() { # lab256|lab512
  local key="$1"
  local host="$(_host_ip ${key})"
  echo "⤴ Host ${key} (${host})"
  local BIN_DIR ETC_DIR LOG_DIR
  BIN_DIR=$(_subst "$(jq -r '.defaults.bin_dir' ${manifest_json})")
  ETC_DIR=$(_subst "$(jq -r '.defaults.etc_dir' ${manifest_json})")
  LOG_DIR=$(_subst "$(jq -r '.defaults.log_dir' ${manifest_json})")

  local count
  count=$(jq ".hosts[\"${key}\"].installs | length" ${manifest_json})
  for ((i=0;i<count;i++)); do
    type=$(jq -r ".hosts[\"${key}\"].installs[$i].type" ${manifest_json})
    case "$type" in
      bin)
        name=$(jq -r ".hosts[\"${key}\"].installs[$i].name" ${manifest_json})
        from_env=$(jq -r ".hosts[\"${key}\"].installs[$i].from_env" ${manifest_json})
        to=$(jq -r ".hosts[\"${key}\"].installs[$i].to" ${manifest_json})
        mode=$(jq -r ".hosts[\"${key}\"].installs[$i].mode" ${manifest_json})
        to=$(_subst "$to"); mode=$(_subst "$mode")
        src="${!from_env:-}"
        if [ -z "${src}" ] || [ ! -f "${src}" ]; then
          echo "⚠️  ${name}: variável ${from_env} não definida ou arquivo não encontrado (${src}). Pulando."
        else
          echo "  ➤ bin ${name} -> ${host}:${to}"
          _push_file "${src}" "${host}" "${to}" "${mode}"
        fi
        ;;
      ecosystem|config)
        name=$(jq -r ".hosts[\"${key}\"].installs[$i].name" ${manifest_json})
        src=$(jq -r ".hosts[\"${key}\"].installs[$i].source // empty" ${manifest_json})
        to=$(jq -r ".hosts[\"${key}\"].installs[$i].to" ${manifest_json})
        to=$(_subst "$to")
        if [ -z "${src}" ] || [ ! -f "../${src}" ]; then
          # fallback: look in ../minis/<key>/
          fallback="../minis/${key}/${name}"
          if [ -f "${fallback}" ]; then src="${fallback}"; else
            echo "⚠️  ${name}: fonte não encontrada (${src}). Pulando."
            continue
          fi
        else
          src="../${src}"
        fi
        echo "  ➤ file ${name} -> ${host}:${to}"
        _push_file "${src}" "${host}" "${to}"
        ;;
      service)
        cmd=$(jq -r ".hosts[\"${key}\"].installs[$i].cmd" ${manifest_json})
        cmd=$(_subst "$cmd")
        echo "  ➤ service ${cmd}"
        _do_service "${host}" "${cmd}"
        ;;
      *)
        echo "❌ Tipo desconhecido: $type"
        exit 1
        ;;
    esac
  done
  echo "✅ publish ${key} ok"
}

case "${1:-}" in
  lab256|lab512) publish_host "$1" ;;
  *)
    echo "Uso: $0 lab256|lab512"
    ;;
esac
