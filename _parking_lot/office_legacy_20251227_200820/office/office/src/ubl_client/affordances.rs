//! Affordances and Obligations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An affordance available to an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UblAffordance {
    /// Affordance ID
    pub id: String,
    /// Affordance name
    pub name: String,
    /// Description
    pub description: String,
    /// Risk score (0.0 to 1.0)
    pub risk_score: f32,
    /// Parameters schema
    pub parameters: Option<serde_json::Value>,
}

/// An obligation for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UblObligation {
    /// Obligation ID
    pub id: String,
    /// Description
    pub description: String,
    /// Due date
    pub due_at: Option<DateTime<Utc>>,
    /// Priority (1-10)
    pub priority: u8,
    /// Source of obligation
    pub source: String,
}
