#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
[ -f .env ] && source .env || true

# Exemplo de build: ajusta conforme seu repo de código
# Espera diretórios: ../code/kernel/rust e ../code/runner (ajuste livre)
if [ -d "../code/kernel/rust" ]; then
  echo "🛠  Build ubl-server (Rust)"
  (cd ../code/kernel/rust && cargo build --release --bin ubl-server)
  export UBL_SERVER_BIN="$(cd ../code/kernel/rust && pwd)/target/release/ubl-server"
  echo "UBL_SERVER_BIN=${UBL_SERVER_BIN}"
fi

if [ -d "../code/runner" ]; then
  echo "🛠  Build ubl-runner (Node/Rust) — ajuste conforme seu runner"
  # placeholder (adicione a sua pipeline de build)
  if [ -f "../code/runner/target/release/ubl-runner" ]; then
    export UBL_RUNNER_BIN="$(cd ../code/runner && pwd)/target/release/ubl-runner"
  fi
  echo "UBL_RUNNER_BIN=${UBL_RUNNER_BIN:-}"
fi

echo "✅ Build(s) concluído(s). Agora rode: bash controller/push_bins.sh lab256 && ... lab512"
