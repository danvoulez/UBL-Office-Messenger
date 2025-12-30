//! # C.Tenant - Tenant Management Container
//!
//! Multi-tenant organization management for UBL.
//!
//! ## Features
//! - Create and manage tenants (organizations)
//! - Invite-based membership
//! - Role-based access (owner, admin, member)
//! - User-tenant binding with default tenant
//!
//! ## Routes
//! - `POST /tenant` - Create new tenant
//! - `GET /tenant` - Get current user's tenant
//! - `GET /tenant/members` - List tenant members
//! - `POST /tenant/invite` - Create invite code
//! - `POST /tenant/join` - Join tenant with invite code

pub mod db;
pub mod routes;
pub mod types;

pub use routes::tenant_routes;
pub use types::*;
