# Cryptography Validation Runbook

## Alert:  Signature Determinism Violation

**Severity**:  CRITICAL  
**Impact**: Cryptographic failure - system compromised

### Immediate Actions (0-5 minutes)

1. **STOP ALL OPERATIONS**
   ```bash
   # Freeze system immediately
   curl -X POST http://localhost:8080/admin/freeze