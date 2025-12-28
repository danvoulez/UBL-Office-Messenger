# C.Office Tests

## Unit Tests

Located in `/office/src/*/tests.rs`

Run with:
```bash
cd office && cargo test
```

## Integration Tests

Test the full flow Office → UBL → Projections:

```bash
# Start UBL Kernel
cd ubl/kernel/rust/ubl-server && cargo run

# In another terminal, run Office tests
cd office && cargo test --features integration
```

## E2E Tests

See `/ubl/tests/e2e/` for end-to-end tests.

