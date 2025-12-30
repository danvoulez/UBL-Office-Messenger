//! # OFFICE - LLM Operating System
//!
//! Runtime for LLM Entities implementing the Universal Historical Specification.
//!
//! ## Sovereignty Model
//!
//! **UBL is the law. Office is an executor subordinate to UBL.**
//!
//! - Office MUST call `/v1/policy/permit` before any mutation
//! - Office CANNOT create identities, ledgers, or parallel policies
//! - Office CAN only restrict more, never allow more than UBL permits
//! - All mutations result in Receipts registered in UBL
//!
//! ## Core Concepts
//!
//! - **Entity**: Persistent LLM identity with cryptographic keys
//! - **Instance**: Ephemeral LLM session that executes work
//! - **Context Frame**: Immutable snapshot of relevant state
//! - **Narrator**: Transforms data into situated narrative
//! - **Handover**: Knowledge transfer between instances
//! - **Constitution**: Behavioral directives that override RLHF (AOP, complementary to UBL)
//! - **Sanity Check**: Validates claims against objective facts
//! - **Dreaming Cycle**: Consolidates memory and removes anxiety
//! - **Permit Middleware**: Enforces UBL sovereignty on all mutations
//!
//! ## Architecture
//!
//! ```text
//!                      UBL (Sovereign)
//!                           │
//!                    ┌──────┴──────┐
//!                    │   Permit    │
//!                    │   (v1.1)    │
//!                    └──────┬──────┘
//!                           │
//! ┌─────────────────────────┼─────────────────────────┐
//! │                      OFFICE                        │
//! │                                                    │
//! │  ┌─────────────────┐                              │
//! │  │ Constitution    │ → AOP: pre-flight, denylists │
//! │  │ Enforcer        │    (can only restrict more)  │
//! │  └────────┬────────┘                              │
//! │           ↓                                        │
//! │  ┌─────────────────┐                              │
//! │  │ Permit          │ → Calls UBL /v1/policy/permit│
//! │  │ Middleware      │    Fail-closed if denied     │
//! │  └────────┬────────┘                              │
//! │           ↓                                        │
//! │  ┌─────────────────┐                              │
//! │  │ Job Executor    │ → LLM + Runner execution     │
//! │  └────────┬────────┘                              │
//! │           ↓                                        │
//! │  ┌─────────────────┐                              │
//! │  │ Receipt         │ → Submitted to UBL           │
//! │  └─────────────────┘                              │
//! └────────────────────────────────────────────────────┘
//! ```

pub mod entity;
pub mod context;
pub mod session;
pub mod governance;
pub mod ubl_client;
pub mod llm;
pub mod api;
pub mod job_executor;
pub mod audit;
pub mod middleware;
pub mod observability;
pub mod types;
pub mod asc;
pub mod routes;
pub mod http_unix;

// Builder function for tests (Prompt 2: Office integration tests)
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

/// Create a simple Office router for testing
/// This is a minimal router that only includes the workspace/deploy routes
pub fn app(ubl_base: String) -> Router {
    let ubl_client = Arc::new(crate::ubl_client::UblClient::with_generated_key(
        &ubl_base,
        "test",
        30000,
    ));
    
    let ws_router = routes::ws::router(routes::ws::OfficeState {
        ubl_base: ubl_base.clone(),
        ubl_client: ubl_client.clone(),
    });
    let deploy_router = routes::deploy::router(routes::deploy::OfficeState {
        ubl_base,
        ubl_client,
    });

    Router::new()
        .merge(ws_router)
        .merge(deploy_router)
        .route("/health", get(|| async { axum::Json(serde_json::json!({"ok":true})) }))
}

// Re-exports for convenience
pub use entity::{Entity, EntityId, Instance, Guardian};
pub use context::{ContextFrame, ContextFrameBuilder, Narrator, Memory};
pub use session::{Session, SessionType, SessionMode, Handover, TokenBudget};
pub use governance::{SanityCheck, Constitution, DreamingCycle, Simulation};
pub use ubl_client::UblClient;
pub use llm::LlmProvider;
pub use job_executor::{JobExecutor, Job, JobId, JobResult, ConversationContext};
pub use middleware::{PermitMiddleware, PermitRequest, PermitResponse, ConstitutionEnforcer};

use thiserror::Error;

/// Core error types for OFFICE
#[derive(Error, Debug)]
pub enum OfficeError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Context building error: {0}")]
    ContextError(String),

    #[error("UBL client error: {0}")]
    UblError(String),

    #[error("LLM provider error: {0}")]
    LlmError(String),

    #[error("Governance error: {0}")]
    GovernanceError(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Job transition error: {0}")]
    JobTransitionError(String),

    #[error("Provenance error: {0}")]
    ProvenanceError(String),

    #[error("Audit error: {0}")]
    AuditError(String),

    #[error("PII policy violation: {0}")]
    PiiViolation(String),

    #[error("Permit denied: {0}")]
    PermitDenied(String),

    #[error("Constitution violation: {0}")]
    ConstitutionViolation(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, OfficeError>;

/// Configuration for OFFICE runtime
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct OfficeConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// UBL connection configuration
    pub ubl: UblConfig,
    /// LLM provider configuration
    pub llm: LlmConfig,
    /// Governance configuration
    pub governance: GovernanceConfig,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UblConfig {
    pub endpoint: String,
    pub container_id: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GovernanceConfig {
    pub sanity_check_enabled: bool,
    pub dreaming_interval_hours: u32,
    pub dreaming_session_threshold: u32,
    pub simulation_required_risk_score: f32,
}

impl Default for OfficeConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_origins: vec!["*".to_string()],
            },
            ubl: UblConfig {
                endpoint: "http://localhost:8080".to_string(),
                container_id: "office".to_string(),
                timeout_ms: 30000,
            },
            llm: LlmConfig {
                provider: "anthropic".to_string(),
                api_key: String::new(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                max_tokens: 4096,
                temperature: 0.7,
            },
            governance: GovernanceConfig {
                sanity_check_enabled: true,
                dreaming_interval_hours: 24,
                dreaming_session_threshold: 50,
                simulation_required_risk_score: 0.7,
            },
        }
    }
}
