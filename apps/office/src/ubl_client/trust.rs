//! Trust Architecture

use serde::{Deserialize, Serialize};

/// Trust level (L0-L5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum TrustLevel {
    /// L0 - Observation only
    L0 = 0,
    /// L1 - Low impact
    L1 = 1,
    /// L2 - Local impact
    L2 = 2,
    /// L3 - Financial impact
    L3 = 3,
    /// L4 - Systemic impact
    L4 = 4,
    /// L5 - Sovereignty/Evolution
    L5 = 5,
}

impl TrustLevel {
    /// Get description for trust level
    pub fn description(&self) -> &'static str {
        match self {
            TrustLevel::L0 => "Observation - Read only, no risk",
            TrustLevel::L1 => "Low impact - Routine operations",
            TrustLevel::L2 => "Local impact - Standard operations",
            TrustLevel::L3 => "Financial impact - Large transfers",
            TrustLevel::L4 => "Systemic impact - Critical operations",
            TrustLevel::L5 => "Sovereignty - Governance changes",
        }
    }

    /// Check if pact is required
    pub fn requires_pact(&self) -> bool {
        *self >= TrustLevel::L3
    }
}

/// Policy chain for trust verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChain {
    /// Chain of policy IDs (from most specific to global)
    pub policies: Vec<PolicyRef>,
    /// Effective trust level
    pub effective_level: TrustLevel,
    /// Whether chain is valid
    pub is_valid: bool,
}

/// Reference to a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRef {
    /// Policy ID
    pub id: String,
    /// Policy version
    pub version: u32,
    /// Trust level
    pub level: TrustLevel,
    /// Whether this policy is pinned
    pub is_pinned: bool,
}

impl PolicyChain {
    /// Create an empty policy chain
    pub fn empty() -> Self {
        Self {
            policies: vec![],
            effective_level: TrustLevel::L0,
            is_valid: false,
        }
    }

    /// Check if chain allows a trust level
    pub fn allows(&self, required: TrustLevel) -> bool {
        self.is_valid && self.effective_level >= required
    }
}
