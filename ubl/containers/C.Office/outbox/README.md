# C.Office Outbox

Office emits:
1. **Events to C.Office** — entity, session, audit events
2. **Jobs to C.Jobs** — via `/messenger/jobs` or direct commit
3. **Commands to Runner** — via `/v1/commands/issue`
4. **Responses to Messenger** — via messages

## Event Flow

```
[LLM Response] → [Constitution Check] → [Sanity Check] → [Commit to Ledger]
                         ↓                    ↓
                 [Block if violation]  [Create governance note]
```

