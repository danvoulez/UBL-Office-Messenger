#!/bin/bash
# Executor: test.run
# Runs project tests

set -e

PARAMS_FILE="$1"
if [ ! -f "$PARAMS_FILE" ]; then
    echo "ERROR: Params file not found: $PARAMS_FILE"
    exit 1
fi

# Parse params
PROJECT_PATH=$(jq -r '.project_path // "."' "$PARAMS_FILE")
TEST_COMMAND=$(jq -r '.test_command // "npm test"' "$PARAMS_FILE")

echo "[test.run] Starting tests"
echo "  Project: $PROJECT_PATH"
echo "  Command: $TEST_COMMAND"

cd "$PROJECT_PATH" 2>/dev/null || {
    echo "ERROR: Project path not found: $PROJECT_PATH"
    exit 1
}

# Run tests
mkdir -p artifacts
$TEST_COMMAND 2>&1 | tee artifacts/test_output.txt

EXIT_CODE=${PIPESTATUS[0]}

echo "[test.run] Completed with exit code $EXIT_CODE"
exit $EXIT_CODE



