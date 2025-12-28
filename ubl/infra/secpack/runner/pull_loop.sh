#!/usr/bin/env bash
set -euo pipefail

BASE="${UBL_API_BASE:-http://10.77.0.1:8080}"
TENANT="${TENANT_ID:-T.UBL}"
RUNNER_ID="${RUNNER_ID:-LAB_512}"

while true; do
  PENDING=$(curl -sf "$BASE/v1/query/commands?tenant_id=$TENANT&pending=1&limit=10" || echo '{"items":[]}')
  echo "$PENDING" | jq -c '.items[]?' | while read -r cmd; do
    JTI=$(echo "$cmd" | jq -r '.jti')
    JOB=$(echo "$cmd" | jq -r '.jobType')

    LOG="/tmp/ubl-runner/logs/${JTI}.log"
    mkdir -p "$(dirname "$LOG")"

    if sandbox-exec -f sandbox/runner.sb /bin/bash -lc "run_job '$JOB' 2>&1 | tee '$LOG'"; then
      STATUS="OK"
      SUMMARY="job $JOB completed"
    else
      STATUS="ERROR"
      SUMMARY="job $JOB failed"
    fi

    LOG_HASH=$(blake3 "$LOG" | awk '{print $1}')

    curl -sf -X POST "$BASE/v1/exec.finish"       -H 'Content-Type: application/json'       -d "$(jq -n --arg permit_id "$JTI" --arg status "$STATUS"                  --arg runner "$RUNNER_ID" --arg logs "$LOG_HASH"                  --arg summary "$SUMMARY"         '{permit_id:$permit_id, status:$status, runner:$runner, logs_hash:$logs, ret:{summary:$summary}}')"       >/dev/null || true
  done

  sleep 2
done
