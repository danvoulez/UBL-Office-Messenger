# C.Office Inbox

Office receives:
1. **Jobs from C.Jobs** — via SSE or polling `/query/jobs`
2. **Messages from C.Messenger** — via SSE
3. **LLM responses** — from providers (Anthropic, OpenAI, etc.)

## SSE Subscription

Office subscribes to:
- `/ledger/C.Jobs/tail` — for new jobs
- `/ledger/C.Messenger/tail` — for conversation updates

