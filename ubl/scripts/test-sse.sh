#!/bin/bash
# Test SSE tail with LISTEN/NOTIFY

# Start SSE stream in background
echo "Starting SSE tail for C.Messenger..."
curl -N http://localhost:8080/ledger/C.Messenger/tail 2>/dev/null &
SSE_PID=$!
sleep 1

# Send a commit while SSE is listening
echo ""
echo "Sending commit seq=3..."
curl -X POST http://localhost:8080/link/commit \
  -H "Content-Type: application/json" \
  -d '{
    "version": 1,
    "container_id": "C.Messenger",
    "expected_sequence": 3,
    "previous_hash": "045cdbe35fd14e24323ad22041fea47ecc67e87eca73f87290c45fbb0d9e5018",
    "atom_hash": "0xijkl9012",
    "intent_class": "Observation",
    "physics_delta": "0",
    "author_pubkey": "0xpubkeyABC",
    "signature": "0xsigDEF"
  }' 2>/dev/null | jq .

sleep 2

# Kill SSE stream
kill $SSE_PID 2>/dev/null

echo ""
echo "✅ Test complete"
