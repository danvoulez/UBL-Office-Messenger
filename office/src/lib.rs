//! Office - LLM Operating System
//!
//! Office is the execution environment for AI agents within UBL.
//! It enforces the Constitution and communicates ONLY via the UBL Gateway.
//!
//! Key principles:
//! - "Office não cria identidades próprias" (Office doesn't create its own identities)
//! - LLMs are workers, not owners
//! - All actions go through permits
//! - No direct database access

pub mod llm;
pub mod middleware;
pub mod ubl_client;

pub use llm::{LlmClient, LlmProvider, Message, MessageRole, CompletionOptions, LlmError};
pub use middleware::constitution;
pub use ubl_client::UblClient;

