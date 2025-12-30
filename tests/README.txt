# Battle Testing Suite

Chaos engineering and resilience validation for the UBL ecosystem. 

## Purpose

- 🔥 Test system resilience under adverse conditions
- 🔥 Validate failure recovery mechanisms
- 🔥 Identify single points of failure
- 🔥 Measure blast radius of failures
- 🔥 Validate SLOs during degradation

## Chaos Experiments

1. **Network Partition**:  Split brain scenarios
2. **Database Failure**: Primary database unavailability
3. **Service Crash**: Unexpected service termination
4. **High Latency**: Slow network conditions
5. **Resource Exhaustion**: CPU/Memory/Disk pressure
6. **Cascading Failure**: Multiple simultaneous failures

## Running Battle Tests

```bash
cd tests/battle-testing
./scripts/run-chaos-suite.sh
```

## Resilience Score

The system is scored on:
- Recovery Time Objective (RTO): <5 minutes
- Recovery Point Objective (RPO): <1 minute
- Availability: >99.9% during chaos
- Data Integrity: 100% (no data loss)
- Graceful Degradation:  Services fail safely

## Safety

- ⚠️  **Never run on production**
- ⚠️  Always use isolated test environment
- ⚠️  Monitor system state during experiments
- ⚠️  Have rollback procedures ready