//! Entity Management Module
//!
//! Manages LLM entities - persistent identities that spawn ephemeral instances.
//! 
//! The "Chair" - A permanent seat at the table for AI entities:
//! - Entity = The Chair (permanent identity, lives in UBL)
//! - Instance = Who sits in the Chair (ephemeral LLM session)

mod entity;
mod instance;
mod identity;
mod guardian;
mod repository;

pub use entity::{Entity, EntityId, EntityParams, EntityType, EntityStatus};
pub use instance::{Instance, InstanceId, InstanceStatus};
pub use identity::{Identity, KeyPair};
pub use guardian::{Guardian, GuardianId};
pub use repository::{EntityRepository, EntityEvent};
