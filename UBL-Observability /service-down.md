# Runbook: Service Down

**Alert**:  `UBLKernelDown` or `OfficeDown`  
**Severity**: Critical  
**SLO Impact**: High

## Symptoms

- Service health check failing
- 100% error rate
- No metrics being reported

## Initial Response (5 minutes)

1. **Acknowledge Alert**
   ```bash
   # In PagerDuty or Slack