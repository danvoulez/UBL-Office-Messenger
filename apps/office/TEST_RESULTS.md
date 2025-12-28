# Test Results

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | 45 |
| Passed | 45 |
| Failed | 0 |
| Ignored | 0 |
| Test Duration | 0.61s |

---

## Test Results by Module

### Context Module

| Test | Status |
|------|--------|
| `context::builder::tests::test_test_builder` | ✅ PASS |
| `context::frame::tests::test_context_frame_creation` | ✅ PASS |
| `context::frame::tests::test_hash_verification` | ✅ PASS |
| `context::memory::tests::test_add_events` | ✅ PASS |
| `context::memory::tests::test_bookmarks` | ✅ PASS |
| `context::memory::tests::test_memory_creation` | ✅ PASS |
| `context::memory::tests::test_token_estimation` | ✅ PASS |
| `context::narrator::tests::test_narrative_generation` | ✅ PASS |

### Entity Module

| Test | Status |
|------|--------|
| `entity::entity::tests::test_create_entity` | ✅ PASS |
| `entity::entity::tests::test_entity_lifecycle` | ✅ PASS |
| `entity::guardian::tests::test_autonomous_guardian` | ✅ PASS |
| `entity::guardian::tests::test_human_guardian` | ✅ PASS |
| `entity::identity::tests::test_identity_generation` | ✅ PASS |
| `entity::identity::tests::test_identity_signing` | ✅ PASS |
| `entity::identity::tests::test_keypair_generation` | ✅ PASS |
| `entity::identity::tests::test_sign_and_verify` | ✅ PASS |
| `entity::instance::tests::test_instance_lifecycle` | ✅ PASS |
| `entity::instance::tests::test_token_budget` | ✅ PASS |

### Governance Module

| Test | Status |
|------|--------|
| `governance::constitution::tests::test_constitution_builder` | ✅ PASS |
| `governance::constitution::tests::test_default_constitution` | ✅ PASS |
| `governance::constitution::tests::test_presets` | ✅ PASS |
| `governance::constitution::tests::test_to_text` | ✅ PASS |
| `governance::dreaming::tests::test_default_config` | ✅ PASS |
| `governance::dreaming::tests::test_is_due_session_threshold` | ✅ PASS |
| `governance::sanity_check::tests::test_check_without_ubl` | ✅ PASS |
| `governance::sanity_check::tests::test_factual_claim_detection` | ✅ PASS |
| `governance::sanity_check::tests::test_keyword_extraction` | ✅ PASS |
| `governance::sanity_check::tests::test_sentiment_estimation` | ✅ PASS |
| `governance::simulation::tests::test_is_required` | ✅ PASS |
| `governance::simulation::tests::test_quick_check` | ✅ PASS |
| `governance::simulation::tests::test_simulate` | ✅ PASS |

### Session Module

| Test | Status |
|------|--------|
| `session::handover::tests::test_handover_builder` | ✅ PASS |
| `session::handover::tests::test_handover_creation` | ✅ PASS |
| `session::handover::tests::test_keyword_extraction` | ✅ PASS |
| `session::modes::tests::test_session_config` | ✅ PASS |
| `session::modes::tests::test_session_mode` | ✅ PASS |
| `session::modes::tests::test_session_type_budgets` | ✅ PASS |
| `session::modes::tests::test_session_type_properties` | ✅ PASS |
| `session::session::tests::test_session_lifecycle` | ✅ PASS |
| `session::session::tests::test_token_budget` | ✅ PASS |
| `session::token_budget::tests::test_budget_tracking` | ✅ PASS |
| `session::token_budget::tests::test_limits` | ✅ PASS |
| `session::token_budget::tests::test_quota_by_entity_type` | ✅ PASS |

### LLM Module

| Test | Status |
|------|--------|
| `llm::local::tests::test_local_provider` | ✅ PASS |

### UBL Client Module

| Test | Status |
|------|--------|
| `ubl_client::tests::test_client_creation` | ✅ PASS |

---

## Coverage Notes

### Well-Covered Areas
- **Entity lifecycle**: Creation, activation, archival
- **Identity management**: Key generation, signing, verification
- **Session management**: Types, modes, configuration
- **Token budgeting**: Allocation, consumption, limits
- **Governance**: Constitution, sanity checks, simulation
- **Memory**: Events, bookmarks, token estimation

### Areas for Future Testing
- HTTP API endpoints (requires mock server setup)
- WebSocket handlers
- Full UBL integration (requires mock UBL server)
- Dreaming cycle execution (requires async test infrastructure)
- LLM provider integration (Anthropic, OpenAI)

---

## Build Information

```
cargo version: 1.75+
Build profile: debug (for testing)
Platform: linux
```

---

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_context_frame_creation

# Run tests for a specific module
cargo test context::
```
