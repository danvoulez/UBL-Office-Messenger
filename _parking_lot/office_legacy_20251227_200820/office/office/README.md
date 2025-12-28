# OFFICE - LLM Operating System

A Rust runtime for LLM Entities implementing the Universal Historical Specification.

## Overview

OFFICE is a standalone LLM runtime that implements all 7 patterns from the Universal Historical Specification:

1. **Context Frame Builder** - Constructs immutable snapshots of relevant state
2. **Narrator** - Transforms data into situated first-person narratives
3. **Session Handover** - Transfers knowledge between ephemeral instances
4. **Sanity Check** - Validates claims against objective facts
5. **Constitution** - Behavioral directives that override RLHF
6. **Dreaming Cycle** - Asynchronous memory consolidation
7. **Safety Net (Simulation)** - Test actions before execution

## Architecture

```
┌─────────────────┐
│ 1. Narrator     │ → Builds narrative from UBL state
│    (Preparation)│    Applies Sanity Check
└────────┬────────┘    Injects Constitution
         ↓
┌─────────────────┐
│ 2. Context      │ → Identity, State, Obligations
│    Frame        │    Capacities, Memory, Affordances
└────────┬────────┘
         ↓
┌─────────────────┐
│ 3. LLM Instance │ → Receives complete frame
│    (Invocation) │    Executes work
└────────┬────────┘    Writes handover
         ↓
┌─────────────────┐
│ 4. Ledger       │ → Registers actions
│    (UBL)        │    Stores receipts
└─────────────────┘    Maintains identity
```

## API Endpoints

### Entities

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/entities` | POST | Create a new entity |
| `/entities` | GET | List all entities |
| `/entities/:id` | GET | Get entity details |
| `/entities/:id` | DELETE | Archive entity |

### Sessions

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/entities/:id/sessions` | POST | Start a new session |
| `/entities/:id/sessions/:sid` | GET | Get session status |
| `/entities/:id/sessions/:sid` | DELETE | End session |
| `/entities/:id/sessions/:sid/message` | POST | Send message to session |

### Governance

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/entities/:id/dream` | POST | Trigger dreaming cycle |
| `/entities/:id/memory` | GET | Get memory state |
| `/entities/:id/constitution` | POST | Update constitution |
| `/entities/:id/constitution` | GET | Get constitution |

### Simulation

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/simulate` | POST | Simulate an action |
| `/affordances` | GET | List available affordances |

### WebSocket

| Endpoint | Description |
|----------|-------------|
| `/entities/:id/stream` | Real-time session updates |

## Session Types

- **Work** - Autonomous work session with full authority (5000 tokens)
- **Assist** - Helping a human with a task (4000 tokens)
- **Deliberate** - Exploring options without committing (8000 tokens)
- **Research** - Gathering information (6000 tokens)

## Session Modes

- **Commitment** - Actions are signed and binding
- **Deliberation** - Actions are drafts, not binding

## Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080
cors_origins = ["*"]

[ubl]
endpoint = "http://localhost:3000"
container_id = "office"
timeout_ms = 30000

[llm]
provider = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-3-5-sonnet-20241022"
max_tokens = 4096
temperature = 0.7

[governance]
sanity_check_enabled = true
dreaming_interval_hours = 24
dreaming_session_threshold = 50
simulation_required_risk_score = 0.7
```

## Running

```bash
# Development
cargo run

# Production
OFFICE__LLM__API_KEY=sk-xxx cargo run --release

# With specific config
cargo run -- --config config/production.toml
```

## Integration with UBL

OFFICE consumes the following from UBL 2.0:

- **Event Sourcing** - Subscribe to ledger events, replay for state
- **Trust Architecture** - Policy chains (L0-L5), pact validation
- **Affordances** - Available actions with risk scores
- **Agreements** - Multi-party commitments
- **Trajectories** - Session history for pattern analysis

## License

MIT
