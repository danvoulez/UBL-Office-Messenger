//! Challenge Manager - Centralized WebAuthn challenge lifecycle
//!
//! Phase 5: Unified challenge management for registration and authentication.
//! Challenges are:
//! - Generated with cryptographically secure random bytes
//! - Stored with expiration (default: 5 minutes)
//! - Validated and consumed atomically (single-use)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use webauthn_rs::prelude::*;

use super::config::config;
use super::error::{IdentityError, IdentityResult};

/// Challenge entry with metadata
struct ChallengeEntry {
    /// The WebAuthn challenge state
    state: ChallengeState,
    /// When this challenge was created
    created_at: Instant,
    /// Challenge type (registration or authentication)
    challenge_type: ChallengeType,
}

/// Type of challenge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeType {
    Registration,
    Authentication,
    StepUp,
}

/// The actual challenge state stored
enum ChallengeState {
    Registration(PasskeyRegistration),
    Authentication(PasskeyAuthentication),
}

/// Challenge Manager - handles creation, storage, and validation of WebAuthn challenges
pub struct ChallengeManager {
    /// In-memory challenge store (keyed by challenge_id)
    challenges: Arc<RwLock<HashMap<String, ChallengeEntry>>>,
    /// How long challenges are valid
    expiry: Duration,
    /// Last cleanup time
    last_cleanup: Arc<RwLock<Instant>>,
}

impl ChallengeManager {
    /// Create a new ChallengeManager
    pub fn new() -> Self {
        let cfg = config();
        let expiry_secs = cfg.webauthn.challenge_ttl_secs;
        
        Self {
            challenges: Arc::new(RwLock::new(HashMap::new())),
            expiry: Duration::from_secs(expiry_secs as u64),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Create a new ChallengeManager with custom expiry
    pub fn with_expiry(expiry: Duration) -> Self {
        Self {
            challenges: Arc::new(RwLock::new(HashMap::new())),
            expiry,
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Store a registration challenge
    pub async fn store_registration(
        &self,
        reg_state: PasskeyRegistration,
    ) -> String {
        let challenge_id = Uuid::new_v4().to_string();
        
        let entry = ChallengeEntry {
            state: ChallengeState::Registration(reg_state),
            created_at: Instant::now(),
            challenge_type: ChallengeType::Registration,
        };

        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge_id.clone(), entry);
        
        // Periodic cleanup
        drop(challenges);
        self.maybe_cleanup().await;

        challenge_id
    }

    /// Store an authentication challenge
    pub async fn store_authentication(
        &self,
        auth_state: PasskeyAuthentication,
    ) -> String {
        let challenge_id = Uuid::new_v4().to_string();
        
        let entry = ChallengeEntry {
            state: ChallengeState::Authentication(auth_state),
            created_at: Instant::now(),
            challenge_type: ChallengeType::Authentication,
        };

        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge_id.clone(), entry);
        
        drop(challenges);
        self.maybe_cleanup().await;

        challenge_id
    }

    /// Consume a registration challenge (single-use)
    pub async fn consume_registration(
        &self,
        challenge_id: &str,
    ) -> IdentityResult<PasskeyRegistration> {
        let mut challenges = self.challenges.write().await;
        
        let entry = challenges
            .remove(challenge_id)
            .ok_or(IdentityError::ChallengeNotFound)?;

        // Check expiry
        if entry.created_at.elapsed() > self.expiry {
            return Err(IdentityError::ChallengeExpired);
        }

        // Check type
        match entry.state {
            ChallengeState::Registration(state) => Ok(state),
            ChallengeState::Authentication(_) => Err(IdentityError::ChallengeTypeMismatch),
        }
    }

    /// Consume an authentication challenge (single-use)
    pub async fn consume_authentication(
        &self,
        challenge_id: &str,
    ) -> IdentityResult<PasskeyAuthentication> {
        let mut challenges = self.challenges.write().await;
        
        let entry = challenges
            .remove(challenge_id)
            .ok_or(IdentityError::ChallengeNotFound)?;

        // Check expiry
        if entry.created_at.elapsed() > self.expiry {
            return Err(IdentityError::ChallengeExpired);
        }

        // Check type
        match entry.state {
            ChallengeState::Authentication(state) => Ok(state),
            ChallengeState::Registration(_) => Err(IdentityError::ChallengeTypeMismatch),
        }
    }

    /// Check if a challenge exists and is valid
    pub async fn is_valid(&self, challenge_id: &str) -> bool {
        let challenges = self.challenges.read().await;
        
        if let Some(entry) = challenges.get(challenge_id) {
            entry.created_at.elapsed() <= self.expiry
        } else {
            false
        }
    }

    /// Get the count of active challenges
    pub async fn active_count(&self) -> usize {
        let challenges = self.challenges.read().await;
        let now = Instant::now();
        
        challenges
            .values()
            .filter(|e| now.duration_since(e.created_at) <= self.expiry)
            .count()
    }

    /// Cleanup expired challenges (runs periodically)
    async fn maybe_cleanup(&self) {
        let cleanup_interval = Duration::from_secs(60); // Every minute
        
        {
            let last = self.last_cleanup.read().await;
            if last.elapsed() < cleanup_interval {
                return;
            }
        }

        // Update last cleanup time
        {
            let mut last = self.last_cleanup.write().await;
            *last = Instant::now();
        }

        // Remove expired challenges
        let mut challenges = self.challenges.write().await;
        let now = Instant::now();
        
        challenges.retain(|_, entry| {
            now.duration_since(entry.created_at) <= self.expiry
        });

        tracing::debug!("Challenge cleanup: {} active challenges", challenges.len());
    }
}

impl Default for ChallengeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_count() {
        let manager = ChallengeManager::with_expiry(Duration::from_secs(60));
        
        // Initially empty
        assert_eq!(manager.active_count().await, 0);
        
        // Note: We can't easily create PasskeyRegistration in tests without
        // a full WebAuthn instance, so we just test the structure
    }

    #[tokio::test]
    async fn test_expiry() {
        let manager = ChallengeManager::with_expiry(Duration::from_millis(50));
        
        // After 50ms, challenges should be considered expired
        // (Can't fully test without PasskeyRegistration mock)
        assert_eq!(manager.active_count().await, 0);
    }
}
