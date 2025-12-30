#!/bin/bash
# Calculate resilience score based on test results

set -e

echo "📊 Calculating Resilience Score..."
echo ""

# Initialize scores
AUTO_RETRY_SCORE=0
CIRCUIT_BREAKER_SCORE=0
GRACEFUL_DEGRADATION_SCORE=0
STATE_RECOVERY_SCORE=0
DATA_INTEGRITY_SCORE=0
AVAILABILITY_SCORE=0

# Check test results (simplified - in real implementation, parse actual test output)
REPORT_FILE="../reports/chaos-test-results.json"

if [ -f "$REPORT_FILE" ]; then
    PASSED=$(grep -o '"passed":  [0-9]*' $REPORT_FILE | cut -d' ' -f2)
    TOTAL=$(grep -o '"experiments": [0-9]*' $REPORT_FILE | cut -d' ' -f2)
    
    if [ -n "$PASSED" ] && [ -n "$TOTAL" ] && [ "$TOTAL" -gt 0 ]; then
        PASS_RATE=$(echo "scale=2; $PASSED * 100 / $TOTAL" | bc)
        
        # Calculate component scores (simplified)
        AUTO_RETRY_SCORE=$(echo "scale=0; $PASS_RATE * 0.15" | bc)
        CIRCUIT_BREAKER_SCORE=$(echo "scale=0; $PASS_RATE * 0.20" | bc)
        GRACEFUL_DEGRADATION_SCORE=$(echo "scale=0; $PASS_RATE * 0.20" | bc)
        STATE_RECOVERY_SCORE=$(echo "scale=0; $PASS_RATE * 0.15" | bc)
        DATA_INTEGRITY_SCORE=$(echo "scale=0; $PASS_RATE * 0.15" | bc)
        AVAILABILITY_SCORE=$(echo "scale=0; $PASS_RATE * 0.15" | bc)
    fi
fi

# Calculate total score
TOTAL_SCORE=$(echo "$AUTO_RETRY_SCORE + $CIRCUIT_BREAKER_SCORE + $GRACEFUL_DEGRADATION_SCORE + $STATE_RECOVERY_SCORE + $DATA_INTEGRITY_SCORE + $AVAILABILITY_SCORE" | bc)

# Display results
echo "═══════════════════════════════════════════════════════"
echo "                  RESILIENCE SCORECARD"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "Category                        Score    Weight"
echo "───────────────────────────────────────────────────────"
echo "Auto-Retry                      $AUTO_RETRY_SCORE/15     15%"
echo "Circuit Breaker                 $CIRCUIT_BREAKER_SCORE/20     20%"
echo "Graceful Degradation            $GRACEFUL_DEGRADATION_SCORE/20     20%"
echo "State Recovery                  $STATE_RECOVERY_SCORE/15     15%"
echo "Data Integrity                  $DATA_INTEGRITY_SCORE/15     15%"
echo "Availability Under Chaos        $AVAILABILITY_SCORE/15     15%"
echo "───────────────────────────────────────────────────────"
echo "TOTAL RESILIENCE SCORE:          $TOTAL_SCORE/100"
echo "═══════════════════════════════════════════════════════"
echo ""

# Grade
if [ "$TOTAL_SCORE" -ge 90 ]; then
    echo "Grade: A+ (Excellent) 🏆"
    echo "The system demonstrates outstanding resilience."
elif [ "$TOTAL_SCORE" -ge 80 ]; then
    echo "Grade:  A (Very Good) ⭐"
    echo "The system shows strong resilience with minor areas for improvement."
elif [ "$TOTAL_SCORE" -ge 70 ]; then
    echo "Grade:  B (Good) ✅"
    echo "The system is resilient but has room for improvement."
elif [ "$TOTAL_SCORE" -ge 60 ]; then
    echo "Grade: C (Acceptable) ⚠️"
    echo "The system needs significant resilience improvements."
else
    echo "Grade: F (Poor) ❌"
    echo "The system requires major resilience enhancements."
fi

echo ""

# Save score
SCORE_FILE="../reports/resilience-score.json"
cat > $SCORE_FILE << EOF
{
  "timestamp": "$(date -Iseconds)",
  "scores": {
    "auto_retry": $AUTO_RETRY_SCORE,
    "circuit_breaker":  $CIRCUIT_BREAKER_SCORE,
    "graceful_degradation": $GRACEFUL_DEGRADATION_SCORE,
    "state_recovery": $STATE_RECOVERY_SCORE,
    "data_integrity": $DATA_INTEGRITY_SCORE,
    "availability": $AVAILABILITY_SCORE
  },
  "total_score": $TOTAL_SCORE,
  "grade": "$([ "$TOTAL_SCORE" -ge 90 ] && echo 'A+' || [ "$TOTAL_SCORE" -ge 80 ] && echo 'A' || [ "$TOTAL_SCORE" -ge 70 ] && echo 'B' || [ "$TOTAL_SCORE" -ge 60 ] && echo 'C' || echo 'F')"
}
EOF

echo "📄 Resilience score saved to: $SCORE_FILE"