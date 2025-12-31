//! Database layer - PostgreSQL ledger with SERIALIZABLE transactions
//! SPEC-UBL-LEDGER v1.0 compliant
//!
//! Fix #12: Includes retry logic for serialization conflicts (SQLSTATE 40001)

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Row, Transaction};
use time::OffsetDateTime;
use tracing::{info, warn};

// Helper trait for getting columns by name (local to this module to avoid conflicts)
trait DbRowExt {
    fn get_col<T>(&self, col: &str) -> T 
    where
        T: for<'r> sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>;
}

impl DbRowExt for sqlx::postgres::PgRow {
    fn get_col<T>(&self, col: &str) -> T 
    where
        T: for<'r> sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        sqlx::Row::get(self, col)
    }
}

#[derive(Debug, Deserialize)]
pub struct LinkDraft {
    pub version: u8,
    pub container_id: String,
    pub expected_sequence: i64,
    pub previous_hash: String,
    pub atom_hash: String,
    pub intent_class: String,     // "Observation"|"Conservation"|"Entropy"|"Evolution"
    pub physics_delta: String,    // i128 string (já validado na Membrane)
    pub author_pubkey: String,    // hex
    pub signature: String,        // hex
    /// The atom data (semantic content) - optional for backward compatibility
    /// but required for projections to work
    #[serde(default)]
    pub atom: Option<serde_json::Value>,
    /// Pact proof (required for Entropy with delta≠0 and Evolution)
    #[serde(default)]
    pub pact: Option<PactProofDraft>,
}

/// Pact proof in link draft
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PactProofDraft {
    pub pact_id: String,
    pub signatures: Vec<PactSignatureDraft>,
}

/// Signature in pact proof
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PactSignatureDraft {
    pub signer: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct LedgerEntry {
    pub container_id: String,
    pub sequence: i64,
    pub link_hash: String,
    pub previous_hash: String,
    pub entry_hash: String,
    pub ts_unix_ms: i64,
}

#[derive(Debug)]
pub enum TangencyError {
    InvalidVersion,
    InvalidTarget,
    RealityDrift,
    SequenceMismatch,
    PactViolation(String),
    /// Fix #12: Serialization conflict (SQLSTATE 40001) - can be retried
    SerializationConflict,
    /// General database error
    DatabaseError(String),
}

#[derive(Clone)]
pub struct PgLedger {
    pool: PgPool,
}

impl PgLedger {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Append transacional com SERIALIZABLE + FOR UPDATE
    /// SPEC-UBL-LEDGER v1.0 §7 - Atomicidade: validate → append → commit
    /// 
    /// Fix #12: Retries up to 3 times on serialization conflict (SQLSTATE 40001)
    pub async fn append(&self, link: &LinkDraft) -> Result<LedgerEntry, TangencyError> {
        const MAX_RETRIES: u32 = 3;
        
        for attempt in 1..=MAX_RETRIES {
            match self.try_append(link).await {
                Ok(entry) => return Ok(entry),
                Err(TangencyError::SerializationConflict) if attempt < MAX_RETRIES => {
                    warn!(
                        "⚠️ Serialization conflict on attempt {}/{} for container {}, retrying...",
                        attempt, MAX_RETRIES, link.container_id
                    );
                    // Brief backoff: 10ms * attempt
                    tokio::time::sleep(std::time::Duration::from_millis(10 * attempt as u64)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(TangencyError::SerializationConflict)
    }

    /// Internal append attempt - may fail with SerializationConflict
    async fn try_append(&self, link: &LinkDraft) -> Result<LedgerEntry, TangencyError> {
        // Begin SERIALIZABLE transaction
        let mut tx: Transaction<Postgres> = self
            .pool
            .begin()
            .await
            .map_err(|e| {
                warn!("Failed to begin transaction: {}", e);
                TangencyError::DatabaseError(e.to_string())
            })?;
        
        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;")
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                warn!("Failed to set isolation level: {}", e);
                TangencyError::DatabaseError(e.to_string())
            })?;

        // Lock and get latest entry (FOR UPDATE)
        let rec: Option<sqlx::postgres::PgRow> = sqlx::query(
            r#"
            SELECT sequence, entry_hash
            FROM ledger_entry
            WHERE container_id = $1
            ORDER BY sequence DESC
            LIMIT 1
            FOR UPDATE
            "#,
        )
        .bind(&link.container_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| Self::classify_error(e))?;

        let (expected_prev, expected_seq) = match rec {
            Some(r) => {
                let entry_hash: String = r.get_col("entry_hash");
                let sequence: i64 = r.get_col("sequence");
                (entry_hash, sequence + 1)
            },
            None => ("0x00".to_string(), 1),
        };

        // Validate causality (SPEC-UBL-MEMBRANE v1.0 §V4)
        if link.previous_hash != expected_prev {
            return Err(TangencyError::RealityDrift);
        }

        // Validate sequence (SPEC-UBL-MEMBRANE v1.0 §V5)
        if link.expected_sequence != expected_seq {
            return Err(TangencyError::SequenceMismatch);
        }

        // Validate version (SPEC-UBL-MEMBRANE v1.0 §V1)
        if link.version != 1 {
            return Err(TangencyError::InvalidVersion);
        }

        // Compute entry_hash per SPEC-UBL-LEDGER v1.0 §5.1
        // entry_hash := BLAKE3("ubl:ledger\n" || container_id || sequence || link_hash || previous_hash || timestamp)
        let ts_unix_ms = (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64;
        let mut h = Hasher::new();
        h.update(b"ubl:ledger\n"); // Domain tag per SPEC-UBL-LEDGER v1.0 §5.1
        h.update(link.container_id.as_bytes());
        let seq_bytes: [u8; 8] = expected_seq.to_be_bytes();
        h.update(&seq_bytes); // Big-endian per spec
        h.update(link.atom_hash.as_bytes()); // link_hash = atom_hash reference
        h.update(expected_prev.as_bytes());
        let ts_bytes: [u8; 8] = ts_unix_ms.to_be_bytes();
        h.update(&ts_bytes); // Big-endian for consistency
        let entry_hash = hex::encode(h.finalize().as_bytes());

        // Insert new entry (SPEC-UBL-LEDGER v1.0 §7.1 - Append-only)
        sqlx::query(
            r#"
            INSERT INTO ledger_entry (container_id, sequence, link_hash, previous_hash, entry_hash, ts_unix_ms, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, '{}'::jsonb)
            "#,
        )
        .bind(&link.container_id)
        .bind(expected_seq)
        .bind(&link.atom_hash)
        .bind(&expected_prev)
        .bind(&entry_hash)
        .bind(ts_unix_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| Self::classify_error(e))?;

        // Store atom data for projections (if provided)
        if let Some(ref atom_data) = link.atom {
            sqlx::query(
                r#"
                INSERT INTO ledger_atom (atom_hash, container_id, atom_data, ts_unix_ms)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (atom_hash) DO NOTHING
                "#,
            )
            .bind(&link.atom_hash)
            .bind(&link.container_id)
            .bind(atom_data)
            .bind(ts_unix_ms)
            .execute(&mut *tx)
            .await
            .map_err(|e| Self::classify_error(e))?;
        }

        // Commit transaction
        tx.commit().await.map_err(|e| Self::classify_error(e))?;

        info!("✅ Ledger append: {} seq={}", link.container_id, expected_seq);

        Ok(LedgerEntry {
            container_id: link.container_id.clone(),
            sequence: expected_seq,
            link_hash: link.atom_hash.clone(),
            previous_hash: expected_prev,
            entry_hash,
            ts_unix_ms,
        })
    }

    /// Classify sqlx errors - detect serialization conflicts (SQLSTATE 40001)
    fn classify_error(e: sqlx::Error) -> TangencyError {
        // Check for PostgreSQL error with SQLSTATE 40001 (serialization_failure)
        if let sqlx::Error::Database(ref db_err) = e {
            if let Some(code) = db_err.code() {
                if code == "40001" {
                    warn!("🔄 Serialization conflict detected (40001)");
                    return TangencyError::SerializationConflict;
                }
            }
        }
        TangencyError::DatabaseError(e.to_string())
    }

    /// Get current state of container
    pub async fn get_state(&self, container_id: &str) -> Result<LedgerEntry, sqlx::Error> {
        let rec: Option<sqlx::postgres::PgRow> = sqlx::query(
            r#"
            SELECT sequence, link_hash, previous_hash, entry_hash, ts_unix_ms
            FROM ledger_entry
            WHERE container_id = $1
            ORDER BY sequence DESC
            LIMIT 1
            "#,
        )
        .bind(container_id)
        .fetch_optional(&self.pool)
        .await?;

        match rec {
            Some(r) => Ok(LedgerEntry {
                container_id: container_id.to_string(),
                sequence: r.get_col("sequence"),
                link_hash: r.get_col("link_hash"),
                previous_hash: r.get_col("previous_hash"),
                entry_hash: r.get_col("entry_hash"),
                ts_unix_ms: r.get_col("ts_unix_ms"),
            }),
            None => Err(sqlx::Error::RowNotFound),
        }
    }
}
