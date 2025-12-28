//! Token Budget - Token Quota Management
//!
//! Manages token quotas and usage tracking for entities and sessions.

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::entity::EntityType;
use super::modes::SessionType;

/// Entity token type - determines quota limits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityTokenType {
    /// Guarded entity - most restrictive
    Guarded,
    /// Autonomous entity - standard limits
    Autonomous,
    /// Development entity - relaxed limits
    Development,
}

impl From<EntityType> for EntityTokenType {
    fn from(et: EntityType) -> Self {
        match et {
            EntityType::Guarded => EntityTokenType::Guarded,
            EntityType::Autonomous => EntityTokenType::Autonomous,
            EntityType::Development => EntityTokenType::Development,
        }
    }
}

/// Token quota configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenQuota {
    /// Daily token limit
    pub daily_limit: u64,
    /// Per-session limits by type
    pub session_limits: SessionTokenLimits,
    /// Soft limit (warning threshold)
    pub soft_limit_percent: f32,
    /// Hard limit (stop threshold)
    pub hard_limit_percent: f32,
}

/// Token limits per session type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTokenLimits {
    pub work: u64,
    pub assist: u64,
    pub deliberate: u64,
    pub research: u64,
}

impl Default for SessionTokenLimits {
    fn default() -> Self {
        Self {
            work: 5000,
            assist: 4000,
            deliberate: 8000,
            research: 6000,
        }
    }
}

impl TokenQuota {
    /// Create quota for entity type
    pub fn for_entity_type(entity_type: EntityTokenType) -> Self {
        match entity_type {
            EntityTokenType::Guarded => Self {
                daily_limit: 50_000,
                session_limits: SessionTokenLimits {
                    work: 3000,
                    assist: 2500,
                    deliberate: 5000,
                    research: 4000,
                },
                soft_limit_percent: 0.8,
                hard_limit_percent: 0.95,
            },
            EntityTokenType::Autonomous => Self {
                daily_limit: 100_000,
                session_limits: SessionTokenLimits::default(),
                soft_limit_percent: 0.8,
                hard_limit_percent: 0.95,
            },
            EntityTokenType::Development => Self {
                daily_limit: 500_000,
                session_limits: SessionTokenLimits {
                    work: 10000,
                    assist: 8000,
                    deliberate: 15000,
                    research: 12000,
                },
                soft_limit_percent: 0.9,
                hard_limit_percent: 0.99,
            },
        }
    }

    /// Get session limit by type
    pub fn get_session_limit(&self, session_type: SessionType) -> u64 {
        match session_type {
            SessionType::Work => self.session_limits.work,
            SessionType::Assist => self.session_limits.assist,
            SessionType::Deliberate => self.session_limits.deliberate,
            SessionType::Research => self.session_limits.research,
        }
    }
}

/// Token budget tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// Entity token type
    pub entity_type: EntityTokenType,
    /// Token quota
    pub quota: TokenQuota,
    /// Tokens used today
    pub daily_used: u64,
    /// Daily reset timestamp
    pub daily_reset_at: DateTime<Utc>,
    /// Tokens used in current session
    pub session_used: u64,
    /// Current session limit
    pub session_limit: u64,
    /// History of usage
    pub usage_history: Vec<UsageRecord>,
}

/// Record of token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub timestamp: DateTime<Utc>,
    pub session_type: SessionType,
    pub tokens_used: u64,
    pub session_id: String,
}

impl TokenBudget {
    /// Create a new budget tracker
    pub fn new(entity_type: EntityTokenType) -> Self {
        let quota = TokenQuota::for_entity_type(entity_type);
        let now = Utc::now();
        let reset_at = now + Duration::days(1);

        Self {
            entity_type,
            quota,
            daily_used: 0,
            daily_reset_at: reset_at,
            session_used: 0,
            session_limit: 0,
            usage_history: Vec::new(),
        }
    }

    /// Start a new session
    pub fn start_session(&mut self, session_type: SessionType) {
        self.check_daily_reset();
        self.session_used = 0;
        self.session_limit = self.quota.get_session_limit(session_type);
    }

    /// Consume tokens
    pub fn consume(&mut self, tokens: u64) {
        self.session_used += tokens;
        self.daily_used += tokens;
    }

    /// Check and perform daily reset if needed
    fn check_daily_reset(&mut self) {
        let now = Utc::now();
        if now >= self.daily_reset_at {
            self.daily_used = 0;
            self.daily_reset_at = now + Duration::days(1);
        }
    }

    /// Get remaining daily budget
    pub fn daily_remaining(&self) -> u64 {
        self.quota.daily_limit.saturating_sub(self.daily_used)
    }

    /// Get remaining session budget
    pub fn session_remaining(&self) -> u64 {
        self.session_limit.saturating_sub(self.session_used)
    }

    /// Check if at soft limit (warning)
    pub fn at_soft_limit(&self) -> bool {
        let daily_percent = self.daily_used as f32 / self.quota.daily_limit as f32;
        daily_percent >= self.quota.soft_limit_percent
    }

    /// Check if at hard limit (stop)
    pub fn at_hard_limit(&self) -> bool {
        let daily_percent = self.daily_used as f32 / self.quota.daily_limit as f32;
        daily_percent >= self.quota.hard_limit_percent
    }

    /// Check if session budget exceeded
    pub fn session_exceeded(&self) -> bool {
        self.session_used >= self.session_limit
    }

    /// Record session completion
    pub fn record_session(&mut self, session_type: SessionType, session_id: String) {
        self.usage_history.push(UsageRecord {
            timestamp: Utc::now(),
            session_type,
            tokens_used: self.session_used,
            session_id,
        });

        // Keep last 100 records
        if self.usage_history.len() > 100 {
            self.usage_history.remove(0);
        }
    }

    /// Get effective remaining budget (minimum of daily and session)
    pub fn effective_remaining(&self) -> u64 {
        std::cmp::min(self.daily_remaining(), self.session_remaining())
    }

    /// Get usage statistics
    pub fn stats(&self) -> BudgetStats {
        BudgetStats {
            daily_limit: self.quota.daily_limit,
            daily_used: self.daily_used,
            daily_remaining: self.daily_remaining(),
            session_limit: self.session_limit,
            session_used: self.session_used,
            session_remaining: self.session_remaining(),
            at_soft_limit: self.at_soft_limit(),
            at_hard_limit: self.at_hard_limit(),
        }
    }
}

/// Budget statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStats {
    pub daily_limit: u64,
    pub daily_used: u64,
    pub daily_remaining: u64,
    pub session_limit: u64,
    pub session_used: u64,
    pub session_remaining: u64,
    pub at_soft_limit: bool,
    pub at_hard_limit: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_by_entity_type() {
        let guarded = TokenQuota::for_entity_type(EntityTokenType::Guarded);
        let autonomous = TokenQuota::for_entity_type(EntityTokenType::Autonomous);
        let development = TokenQuota::for_entity_type(EntityTokenType::Development);

        assert!(guarded.daily_limit < autonomous.daily_limit);
        assert!(autonomous.daily_limit < development.daily_limit);
    }

    #[test]
    fn test_budget_tracking() {
        let mut budget = TokenBudget::new(EntityTokenType::Autonomous);

        budget.start_session(SessionType::Work);
        assert_eq!(budget.session_limit, 5000);

        budget.consume(1000);
        assert_eq!(budget.session_used, 1000);
        assert_eq!(budget.daily_used, 1000);
        assert_eq!(budget.session_remaining(), 4000);
    }

    #[test]
    fn test_limits() {
        let mut budget = TokenBudget::new(EntityTokenType::Guarded);
        budget.quota.daily_limit = 100;
        budget.quota.soft_limit_percent = 0.8;
        budget.quota.hard_limit_percent = 0.95;

        budget.start_session(SessionType::Work);

        budget.consume(70);
        assert!(!budget.at_soft_limit());

        budget.consume(15);
        assert!(budget.at_soft_limit());
        assert!(!budget.at_hard_limit());

        budget.consume(15);
        assert!(budget.at_hard_limit());
    }
}
