# C.Office Local

Local state managed by Office runtime (not in UBL ledger):

## In-Memory
- Active sessions (ephemeral)
- LLM conversation buffers
- Token counting

## Derived from Ledger (Projections)
- Entity state (from `entity.*` events)
- Constitution (from `constitution.*` events)
- Handover history (from `session.completed` events)

