# C.Office Projections

Projections derived from C.Office events in the ledger.

## Available Queries

| Endpoint | Description |
|----------|-------------|
| `GET /query/office/entities` | List all entities |
| `GET /query/office/entities/:id` | Get entity by ID |
| `GET /query/office/entities/:id/handovers` | Get handover history |
| `GET /query/office/entities/:id/sessions` | Get session history |
| `GET /query/office/entities/:id/constitution` | Get current constitution |
| `GET /query/office/audit?entity_id=X` | Get audit trail |

## Implementation

Projections are implemented in the UBL Kernel under:
`ubl/kernel/rust/ubl-server/src/projections/office.rs`

