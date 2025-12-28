//! Middleware Module
//!
//! AOP (Aspect-Oriented Programming) for Office operations.
//! Enforces UBL sovereignty: all mutations require Permit from UBL.

mod permit;
mod constitution;

pub use permit::{PermitMiddleware, PermitRequest, PermitResponse, PermitError};
pub use constitution::{ConstitutionEnforcer, OfficeConstitution};

