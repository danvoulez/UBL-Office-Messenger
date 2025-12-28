//! Audit Trail Module
//!
//! Makes LLM work auditable and reconstructible.
//! Every tool call, every result, every decision - recorded forever.
//!
//! "If it's not in the ledger, it didn't happen."

mod tool_audit;
mod pii;
mod events;

pub use tool_audit::{ToolAudit, ToolCall, ToolResult, ToolError};
pub use pii::{PiiPolicy, redact_email, redact_phone, hash_pii};
pub use events::{AuditEvent, AuditEventType};

