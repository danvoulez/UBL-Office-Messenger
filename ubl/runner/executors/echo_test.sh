#!/bin/bash
# echo_test executor - Simple test job
set -e

PARAMS_FILE="$1"

echo "=== Echo Test Executor ==="
echo "Command ID: $COMMAND_ID"
echo "Action: $ACTION"
echo "Office: $OFFICE"
echo "Target: $TARGET"
echo "Risk: $RISK"
echo ""
echo "Params:"
cat "$PARAMS_FILE"
echo ""

# Simulate some work
sleep 1

# Write output
cat > "$OUTPUT_FILE" << EOF
{
  "success": true,
  "message": "Echo test completed successfully",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "params_received": $(cat "$PARAMS_FILE")
}
EOF

echo "=== Done ==="

