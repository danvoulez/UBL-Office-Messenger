#!/bin/bash
# Executor: llm.inference.run
# Runs an LLM inference job via Ollama

set -e

PARAMS_FILE="$1"
if [ ! -f "$PARAMS_FILE" ]; then
    echo "ERROR: Params file not found: $PARAMS_FILE"
    exit 1
fi

# Parse params
MODEL=$(jq -r '.model // "llama2"' "$PARAMS_FILE")
PROMPT=$(jq -r '.prompt // ""' "$PARAMS_FILE")
MAX_TOKENS=$(jq -r '.max_tokens // 1024' "$PARAMS_FILE")

echo "[llm.inference.run] Starting inference"
echo "  Model: $MODEL"
echo "  Max tokens: $MAX_TOKENS"

# Call Ollama API
RESPONSE=$(curl -s http://localhost:11434/api/generate \
    -d "{\"model\": \"$MODEL\", \"prompt\": \"$PROMPT\", \"stream\": false}")

# Save output
mkdir -p artifacts
echo "$RESPONSE" | jq -r '.response // ""' > artifacts/output.txt
echo "$RESPONSE" | jq '{model, created_at, done, total_duration, prompt_eval_count, eval_count}' > artifacts/metrics.json

echo "[llm.inference.run] Completed"
echo "  Output saved to artifacts/output.txt"



