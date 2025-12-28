//! # Rate Limiting
//!
//! Simple in-memory rate limiter for WebAuthn endpoints with progressive lockout

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;

#[derive(Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    /// Map of key -> (count, window_start)
    buckets: HashMap<String, (u32, OffsetDateTime)>,
    /// Map of key -> failure state for progressive lockout
    failures: HashMap<String, FailState>,
}

#[derive(Clone, Default)]
pub struct FailState {
    pub fails: u32,
    pub last_fail_epoch: i64,
}

impl FailState {
    pub fn penalty_secs(&self) -> u64 {
        if self.fails <= 5 {
            0
        } else {
            // Exponential backoff: 2^(fails-5) * 60 seconds
            // Capped at 8 to prevent overflow (2^8 * 60 = 256 minutes)
            2u64.pow((self.fails - 5).min(8)) * 60
        }
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimitState {
                buckets: HashMap::new(),
                failures: HashMap::new(),
            })),
        }
    }

    /// Check if request is allowed. Returns Ok(()) if allowed, Err(retry_after_secs) if rate limited.
    pub fn check(&self, key: &str, max_requests: u32, window_secs: i64) -> Result<(), u64> {
        let mut state = self.state.lock().unwrap();
        let now = OffsetDateTime::now_utc();
        
        // Check progressive lockout first
        if let Some(fail_state) = state.failures.get(key) {
            let penalty = fail_state.penalty_secs();
            if penalty > 0 {
                let elapsed = now.unix_timestamp() - fail_state.last_fail_epoch;
                if elapsed < penalty as i64 {
                    return Err((penalty as i64 - elapsed) as u64);
                }
            }
        }
        
        // Clean up expired entries periodically
        state.buckets.retain(|_, (_, start)| {
            now - *start < time::Duration::seconds(window_secs * 2)
        });

        let entry = state.buckets.entry(key.to_string()).or_insert((0, now));
        
        // Reset window if expired
        if now - entry.1 >= time::Duration::seconds(window_secs) {
            *entry = (1, now);
            return Ok(());
        }

        // Check limit
        if entry.0 >= max_requests {
            let retry_after = (entry.1 + time::Duration::seconds(window_secs) - now).whole_seconds() as u64;
            return Err(retry_after);
        }

        entry.0 += 1;
        Ok(())
    }
    
    /// Record a failed authentication attempt for progressive lockout
    pub fn on_fail(&self, key: &str) {
        let mut state = self.state.lock().unwrap();
        let now = OffsetDateTime::now_utc().unix_timestamp();
        
        let fail_state = state.failures.entry(key.to_string()).or_insert(FailState::default());
        fail_state.fails = fail_state.fails.saturating_add(1);
        fail_state.last_fail_epoch = now;
    }
    
    /// Reset failure counter on successful authentication
    pub fn on_success(&self, key: &str) {
        let mut state = self.state.lock().unwrap();
        state.failures.remove(key);
    }
    
    /// Get current failure state for debugging/logging
    pub fn get_failures(&self, key: &str) -> u32 {
        let state = self.state.lock().unwrap();
        state.failures.get(key).map(|f| f.fails).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit() {
        let limiter = RateLimiter::new();
        
        // Allow first 3 requests
        assert!(limiter.check("user1", 3, 60).is_ok());
        assert!(limiter.check("user1", 3, 60).is_ok());
        assert!(limiter.check("user1", 3, 60).is_ok());
        
        // Block 4th request
        assert!(limiter.check("user1", 3, 60).is_err());
    }
}
