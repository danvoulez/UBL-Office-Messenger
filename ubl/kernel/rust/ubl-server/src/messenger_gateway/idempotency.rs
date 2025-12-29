//! Idempotency Management
//!
//! Prevents duplicate actions using tenant-scoped idempotency keys.
//! Format: `idem:{tenant_id}:{action_type}:{resource_id}:{nonce}`

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Idempotency store (in-memory for now, can be moved to Redis)
pub struct IdempotencyStore {
    store: Arc<RwLock<HashMap<String, IdempotencyRecord>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    pub status: String, // "pending", "completed", "failed"
    pub response_body: Option<serde_json::Value>,
    pub created_event_ids: Vec<String>,
    pub created_at: OffsetDateTime,
}

impl IdempotencyStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if idempotency key exists and return cached response
    pub fn check(&self, key: &str) -> Option<IdempotencyRecord> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    /// Store idempotency record
    pub fn store(&self, key: String, record: IdempotencyRecord) {
        let mut store = self.store.write().unwrap();
        store.insert(key, record);
    }

    /// Generate idempotency key
    pub fn generate_key(
        tenant_id: &str,
        action_type: &str,
        resource_id: &str,
        nonce: &str,
    ) -> String {
        format!("idem:{}:{}:{}:{}", tenant_id, action_type, resource_id, nonce)
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

