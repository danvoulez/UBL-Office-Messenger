//! Policy Pack v1 Policies
//!
//! Implements specific policy checks: FSM validation, card provenance, etc.

use serde_json::Value;
use sqlx::PgPool;
use tracing::{error, info, warn};

/// Policy engine for validating events
pub struct PolicyEngine {
    pool: PgPool,
}

#[derive(Debug)]
pub enum PolicyError {
    IllegalJobTransition { from: String, to: String },
    InvalidProvenance { reason: String },
    RawPiiDetected { field: String },
    ToolPairingViolation { tool_call_id: String },
    TenantViolation { reason: String },
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::IllegalJobTransition { from, to } => {
                write!(f, "Illegal job transition: {} → {}", from, to)
            }
            PolicyError::InvalidProvenance { reason } => {
                write!(f, "Invalid card provenance: {}", reason)
            }
            PolicyError::RawPiiDetected { field } => {
                write!(f, "Raw PII detected in field: {}", field)
            }
            PolicyError::ToolPairingViolation { tool_call_id } => {
                write!(f, "Tool result without prior call: {}", tool_call_id)
            }
            PolicyError::TenantViolation { reason } => {
                write!(f, "Tenant violation: {}", reason)
            }
        }
    }
}

impl std::error::Error for PolicyError {}

pub type PolicyResult<T> = Result<T, PolicyError>;

impl PolicyEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Validate job state transition
    pub async fn validate_job_fsm(
        &self,
        job_id: &str,
        from_state: &str,
        to_state: &str,
    ) -> PolicyResult<()> {
        // Allowed transitions (from spec)
        let allowed: std::collections::HashMap<&str, Vec<&str>> = [
            ("draft", vec!["proposed"]),
            ("proposed", vec!["approved", "rejected"]),
            ("approved", vec!["in_progress"]),
            ("in_progress", vec!["waiting_input", "completed", "failed", "cancelled"]),
            ("waiting_input", vec!["in_progress", "cancelled", "failed"]),
            ("completed", vec![]),
            ("rejected", vec![]),
            ("cancelled", vec![]),
            ("failed", vec![]),
        ]
        .iter()
        .cloned()
        .collect();

        // Check if transition is allowed
        if let Some(allowed_states) = allowed.get(from_state) {
            if !allowed_states.contains(&to_state) {
                error!("❌ Illegal job transition: {} → {} (job: {})", from_state, to_state, job_id);
                return Err(PolicyError::IllegalJobTransition {
                    from: from_state.to_string(),
                    to: to_state.to_string(),
                });
            }
        } else {
            warn!("⚠️ Unknown from_state: {}", from_state);
        }

        info!("✅ Job FSM transition valid: {} → {} (job: {})", from_state, to_state, job_id);
        Ok(())
    }

    /// Validate card provenance
    pub async fn validate_card_provenance(
        &self,
        card_id: &str,
        button_id: &str,
        action_type: &str,
    ) -> PolicyResult<()> {
        // Query for prior message.sent event with this card_id
        let card_exists = sqlx::query!(
            r#"
            SELECT 1
            FROM ledger_atom la
            JOIN ledger_entry le ON la.hash = le.link_hash
            WHERE le.container_id = 'C.Messenger'
              AND la.data->>'type' = 'message.sent'
              AND la.data->'payload'->'card'->>'card_id' = $1
            LIMIT 1
            "#,
            card_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error checking card provenance: {}", e);
            PolicyError::InvalidProvenance {
                reason: format!("Database error: {}", e),
            }
        })?;

        if card_exists.is_none() {
            return Err(PolicyError::InvalidProvenance {
                reason: format!("Card {} not found in prior message.sent event", card_id),
            });
        }

        // TODO: Verify button_id exists in card and action_type matches
        // For now, just check card exists

        info!("✅ Card provenance valid: card={} button={}", card_id, button_id);
        Ok(())
    }

    /// Check for raw PII
    pub fn check_no_raw_pii(&self, atom: &Value) -> PolicyResult<()> {
        // Simple check - look for common PII patterns
        let atom_str = serde_json::to_string(atom).unwrap_or_default().to_lowercase();
        
        let pii_patterns = [
            (r"\b\d{3}-\d{2}-\d{4}\b", "ssn"),
            (r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b", "credit_card"),
            (r"\b\d{3}-\d{3}-\d{4}\b", "phone"),
        ];

        for (pattern, field_type) in pii_patterns.iter() {
            if regex::Regex::new(pattern).unwrap().is_match(&atom_str) {
                return Err(PolicyError::RawPiiDetected {
                    field: field_type.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate tool pairing (tool.called before tool.result)
    pub async fn validate_tool_pairing(
        &self,
        tool_call_id: &str,
        event_type: &str,
    ) -> PolicyResult<()> {
        if event_type == "tool.result" {
            // Check if tool.called exists for this tool_call_id
            let called_exists = sqlx::query!(
                r#"
                SELECT 1
                FROM ledger_atom la
                JOIN ledger_entry le ON la.hash = le.link_hash
                WHERE le.container_id = 'C.Jobs'
                  AND la.data->>'type' = 'tool.called'
                  AND la.data->'payload'->>'tool_call_id' = $1
                LIMIT 1
                "#,
                tool_call_id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!("Database error checking tool pairing: {}", e);
                PolicyError::ToolPairingViolation {
                    tool_call_id: tool_call_id.to_string(),
                }
            })?;

            if called_exists.is_none() {
                return Err(PolicyError::ToolPairingViolation {
                    tool_call_id: tool_call_id.to_string(),
                });
            }
        }

        Ok(())
    }
}

