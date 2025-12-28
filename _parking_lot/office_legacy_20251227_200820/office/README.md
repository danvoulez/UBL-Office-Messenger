# Office

A complete implementation of an LLM-powered application stack built on the Universal Blockchain Ledger (UBL) event sourcing system.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         APPLICATIONS                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   UBL Messenger                        │    │
│  │   • Conversations    • Messages    • Smart Features     │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
├─────────────────────────────────────────────────────────────────┤
│                        LLM RUNTIME                               │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                         Office                           │    │
│  │   • Entity Management    • Context Frames                │    │
│  │   • Session Handling     • Constitution                  │    │
│  │   • Sanity Checking      • Dreaming Cycle               │    │
│  │   • Simulation           • Narrator                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
├─────────────────────────────────────────────────────────────────┤
│                       EVENT SOURCING                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                         UBL                              │    │
│  │   • Ledger State         • Link Commits                  │    │
│  │   • Affordances          • Trust Levels (L0-L5)         │    │
│  │   • Event Streams        • Cryptographic Proofs         │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### UBL (Universal Blockchain Ledger)

The event sourcing backbone providing:
- **Immutable Ledger**: All events are cryptographically chained
- **Trust Levels**: L0 (Observation) through L5 (Sovereignty)
- **Affordances**: Dynamic capability grants
- **Event Streams**: Real-time event subscriptions

### Office (LLM Operating System)

The LLM runtime implementing the Universal Historical Specification:

| Pattern | Description |
|---------|-------------|
| **Context Frame Builder** | Assembles immutable snapshots of relevant state |
| **Narrator** | Transforms structured data into first-person narratives |
| **Session Handover** | Manages continuity across LLM interactions |
| **Sanity Check** | Validates claims against objective facts |
| **Constitution** | Behavioral directives that override base training |
| **Dreaming Cycle** | Asynchronous memory consolidation and pattern synthesis |
| **Simulation** | Predicts outcomes before committing actions |

### Messenger

A messaging application demonstrating OFFICE integration:
- Conversations with human and LLM participants
- Smart features (summarization, sentiment, suggestions)
- Full audit trail in UBL ledger
- Message cost tracking and cryptographic signing

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Anthropic API key (for LLM features)

### Running

```bash
# Clone and enter directory
cd office

# Set API key
export ANTHROPIC_API_KEY=your-api-key

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Check health
curl http://localhost:3000/health  # UBL
curl http://localhost:8080/health  # OFFICE
curl http://localhost:8081/health  # Messenger
```

### Development

```bash
# Run UBL
cd UBL-Containers-main && cargo run

# Run Office (in new terminal)
cd office && cargo run

# Run UBL Messenger (in new terminal)
cd ../messenger/backend && cargo run
```

## API Overview

### Office API (port 8080)

```
POST   /entities              Create entity
GET    /entities/:id          Get entity
POST   /entities/:id/session  Start session
POST   /sessions/:id/message  Send message
POST   /sessions/:id/end      End session
POST   /entities/:id/dream    Trigger dreaming
POST   /sanity/check          Validate claims
POST   /simulate              Simulate action
```

### UBL Messenger API (port 8081)

```
POST   /conversations                    Create conversation
GET    /conversations                    List conversations
POST   /conversations/:id/messages       Send message
GET    /conversations/:id/messages       Get messages
POST   /conversations/:id/llm            Add LLM participant
POST   /conversations/:id/summarize      Summarize conversation
POST   /conversations/:id/extract-actions Extract action items
```

### UBL API (port 3000)

```
GET    /health                           Health check
GET    /state/:container                 Get ledger state
POST   /link/commit                      Commit new link
GET    /ledger/:container/events         Get events
GET    /affordances/:container/:entity   Get affordances
```

## Trust Architecture

The system implements UBL's trust levels:

| Level | Name | Description |
|-------|------|-------------|
| L0 | Observation | Read-only, can observe events |
| L1 | Participation | Can participate in existing flows |
| L2 | Conservation | Can preserve and maintain state |
| L3 | Entropy | Can deprecate and remove |
| L4 | Generation | Can create new structures |
| L5 | Sovereignty | Full autonomy, can evolve rules |

## Entity Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│                     Entity Lifecycle                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────┐    ┌─────────┐    ┌──────────┐    ┌──────────┐ │
│  │ Create │───▶│ Session │───▶│ Interact │───▶│ Handover │ │
│  └────────┘    └─────────┘    └──────────┘    └──────────┘ │
│       │             │               │               │        │
│       ▼             ▼               ▼               ▼        │
│  ┌────────┐    ┌─────────┐    ┌──────────┐    ┌──────────┐ │
│  │Identity│    │ Context │    │Governance│    │ Dreaming │ │
│  │  Keys  │    │  Frame  │    │  Checks  │    │  Cycle   │ │
│  └────────┘    └─────────┘    └──────────┘    └──────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Memory Architecture

OFFICE implements a hybrid memory strategy:

1. **Verbatim Recent**: Last N interactions preserved exactly
2. **Historical Synthesis**: Compressed summaries of older context
3. **Bookmarks**: User-marked important moments
4. **Baseline Narrative**: Entity's foundational understanding

## Configuration

See individual component configuration files:
- `office/config/development.toml`
- `office/config/production.toml`
- `messenger/config/development.toml`
- `messenger/config/production.toml`

## License

MIT

## References

- [Universal Historical Specification](./UNIVERSAL-HISTORICAL-SPECIFICATION.md)
- [UBL Discovery Notes](./DISCOVERY.md)
- [OFFICE Documentation](./office/README.md)
- [Messenger Documentation](./messenger/README.md)
