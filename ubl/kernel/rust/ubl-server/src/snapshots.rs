//! Projection Snapshots (Gemini P1 #4)
//!
//! Problem: UBL-Native apps rebuild state from ledger on restart.
//! With 1M events, this takes minutes (unacceptable downtime).
//!
//! Solution: Periodic snapshots + incremental replay from last_sequence.
//!
//! Storage: JSON files in <data_dir>/snapshots/<container_id>.json

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn, error};

/// Snapshot metadata + state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<T> {
    pub container_id: String,
    pub last_sequence: i64,
    pub created_at_ms: i64,
    pub entry_hash: String,
    pub state: T,
}

/// Snapshots directory
fn snapshots_dir() -> PathBuf {
    let base = std::env::var("UBL_DATA_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.ubl/data", home)
        });
    PathBuf::from(base).join("snapshots")
}

/// Initialize snapshots directory
pub fn init() {
    let dir = snapshots_dir();
    if !dir.exists() {
        if let Err(e) = fs::create_dir_all(&dir) {
            error!("Failed to create snapshots directory {:?}: {}", dir, e);
        } else {
            info!("Created snapshots directory: {:?}", dir);
        }
    }
}

/// Save a snapshot for a container
pub fn save<T: Serialize>(
    container_id: &str,
    last_sequence: i64,
    entry_hash: &str,
    state: &T,
) -> Result<(), String> {
    let snapshot = Snapshot {
        container_id: container_id.to_string(),
        last_sequence,
        created_at_ms: now_ms(),
        entry_hash: entry_hash.to_string(),
        state,
    };
    
    let path = snapshots_dir().join(format!("{}.json", container_id));
    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|e| format!("Serialize error: {}", e))?;
    
    fs::write(&path, &json)
        .map_err(|e| format!("Write error: {}", e))?;
    
    info!(
        "📸 Snapshot saved: {} @ seq {} ({} bytes)",
        container_id,
        last_sequence,
        json.len()
    );
    
    Ok(())
}

/// Load a snapshot for a container
pub fn load<T: for<'de> Deserialize<'de>>(container_id: &str) -> Option<Snapshot<T>> {
    let path = snapshots_dir().join(format!("{}.json", container_id));
    
    if !path.exists() {
        info!("No snapshot found for {}, will replay from genesis", container_id);
        return None;
    }
    
    match fs::read_to_string(&path) {
        Ok(json) => {
            match serde_json::from_str(&json) {
                Ok(snapshot) => {
                    let s: Snapshot<T> = snapshot;
                    info!(
                        "📸 Snapshot loaded: {} @ seq {} (age: {}s)",
                        container_id,
                        s.last_sequence,
                        (now_ms() - s.created_at_ms) / 1000
                    );
                    Some(s)
                }
                Err(e) => {
                    warn!("Invalid snapshot for {}: {}", container_id, e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Could not read snapshot for {}: {}", container_id, e);
            None
        }
    }
}

/// Get events since a sequence for replay
pub async fn get_events_since(
    pool: &PgPool,
    container_id: &str,
    since_sequence: i64,
) -> Vec<LedgerEvent> {
    let rows = sqlx::query(
        r#"
        SELECT 
            e.sequence,
            e.entry_hash,
            e.link_hash,
            e.previous_hash,
            e.ts_unix_ms,
            a.data as atom
        FROM ledger_entry e
        LEFT JOIN ledger_atom a ON e.link_hash = a.hash
        WHERE e.container_id = $1 AND e.sequence > $2
        ORDER BY e.sequence ASC
        "#,
    )
    .bind(container_id)
    .bind(since_sequence)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    use sqlx::Row;
    
    rows.into_iter()
        .map(|r| LedgerEvent {
            sequence: r.get("sequence"),
            entry_hash: r.get("entry_hash"),
            link_hash: r.get("link_hash"),
            previous_hash: r.get("previous_hash"),
            ts_unix_ms: r.get("ts_unix_ms"),
            atom: r.get::<Option<serde_json::Value>, _>("atom"),
        })
        .collect()
}

/// Ledger event for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEvent {
    pub sequence: i64,
    pub entry_hash: String,
    pub link_hash: String,
    pub previous_hash: String,
    pub ts_unix_ms: i64,
    pub atom: Option<serde_json::Value>,
}

/// Snapshot interval: save every N events
pub const SNAPSHOT_INTERVAL: i64 = 1000;

/// Check if we should save a snapshot (every SNAPSHOT_INTERVAL events)
pub fn should_snapshot(current_sequence: i64) -> bool {
    current_sequence > 0 && current_sequence % SNAPSHOT_INTERVAL == 0
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// =============================================================================
// JOBS PROJECTION STATE (for snapshot)
// =============================================================================

/// Jobs projection state for snapshotting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobsProjectionState {
    pub jobs: std::collections::HashMap<String, JobState>,
    pub approvals: std::collections::HashMap<String, ApprovalState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub id: String,
    pub conversation_id: String,
    pub entity_id: String,
    pub job_type: String,
    pub status: String,
    pub percent_complete: i32,
    pub created_at_ms: i64,
    pub completed_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalState {
    pub id: String,
    pub job_id: String,
    pub status: String,
    pub created_at_ms: i64,
    pub decided_at_ms: Option<i64>,
}

// =============================================================================
// MESSENGER PROJECTION STATE (for snapshot)
// =============================================================================

/// Messenger projection state for snapshotting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessengerProjectionState {
    pub conversations: std::collections::HashMap<String, ConversationState>,
    pub messages_count: std::collections::HashMap<String, i64>, // conversation_id -> count
    pub last_message_ts: std::collections::HashMap<String, i64>, // conversation_id -> ts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub id: String,
    pub title: String,
    pub participants: Vec<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

