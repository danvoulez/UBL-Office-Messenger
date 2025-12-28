#!/usr/bin/env bash
set -euo pipefail
brew list minio >/dev/null 2>&1 || brew install minio
brew services start minio
echo "✅ MinIO iniciado (ajuste credenciais depois)"
