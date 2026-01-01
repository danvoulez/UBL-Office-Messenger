//! # Identity Module
//!
//! Centralized identity management for UBL.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     identity/                               │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  config.rs       - Centralized configuration (WebAuthn,    │
//! │                    Session, RateLimit)                      │
//! │                                                             │
//! │  error.rs        - Unified error types with IntoResponse   │
//! │                                                             │
//! │  challenge.rs    - ChallengeManager for WebAuthn flows     │
//! │                    (in-memory, no database required)        │
//! │                                                             │
//! │  session.rs      - SessionManager types                     │
//! │                    (will use existing id_session table)     │
//! │                                                             │
//! │  token.rs        - TokenManager types for ASC               │
//! │                    (will use existing id_asc table)         │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Phase 5 Status
//!
//! - ✅ ChallengeManager: Ready (in-memory challenge store)
//! - ⏳ SessionManager: Types defined, DB integration pending
//! - ⏳ TokenManager: Types defined, DB integration pending
//!
//! These will be fully integrated when the existing id_routes.rs
//! is refactored to use these centralized services.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ubl_server::identity::{config, IdentityError, IdentityResult};
//! use ubl_server::identity::ChallengeManager;
//!
//! // Get configuration
//! let origin = config().webauthn.origin.clone();
//!
//! // Return typed errors
//! fn my_handler() -> IdentityResult<()> {
//!     Err(IdentityError::UserNotFound("test".into()))
//! }
//! ```

mod config;
mod error;
mod challenge;
// Session and token types are defined but DB-dependent methods
// are commented out until migration from id_routes.rs
// mod session;
// mod token;

// ============================================================================
// Public API
// ============================================================================

// Configuration
pub use config::{
    config,
    IdentityConfig,
    WebAuthnConfig,
    SessionConfig,
    RateLimitConfig,
};

// Errors
pub use error::{
    IdentityError,
    IdentityResult,
};

// Challenge Management (Phase 5 - Ready)
pub use challenge::{
    ChallengeManager,
    ChallengeType,
};

// Session and Token types will be added when integrating with existing tables:
// - SessionManager: will wrap auth/session_db.rs
// - TokenManager: will wrap id_db.rs ASC functions
