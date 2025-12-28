//! Office Middleware
//!
//! Constitution enforcement and request validation

pub mod constitution;

pub use constitution::{enforce, validate_outbound_url, ConstitutionError};

