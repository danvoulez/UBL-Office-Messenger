#!/usr/bin/env bash
# Configure an 'mc' alias called 'ubl' and create buckets.
set -euo pipefail
ENDPOINT="${ENDPOINT:-http://127.0.0.1:9000}"
ACCESS="${ACCESS:?MINIO ACCESS KEY}"
SECRET="${SECRET:?MINIO SECRET KEY}"
mc alias set ubl "$ENDPOINT" "$ACCESS" "$SECRET" --api S3v4
mc mb -p ubl/vault-repos || true
mc mb -p ubl/vault-workspaces || true
mc mb -p ubl/vault-deploy || true
echo "Alias 'ubl' set. Buckets ensured."
