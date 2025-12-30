//! UBL Client Module
//!
//! HTTP client for interacting with UBL 2.0 ledger.
//! Properly signs all commits using Ed25519.

mod ledger;
mod affordances;
mod receipts;
mod events;
mod trust;
mod identity_events;

pub use ledger::{LedgerState, LedgerEvent};
pub use affordances::{UblAffordance, UblObligation};
pub use receipts::Receipt;
pub use events::EventStream;
pub use trust::{TrustLevel, PolicyChain};
pub use identity_events::{IdentityEvent, IdentityEventKind, IDENTITY_CONTAINER};

use std::time::Duration;

use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::entity::EntityId;
use crate::session::Handover;
use crate::{OfficeError, Result};

/// UBL Client for ledger operations with Ed25519 signing
pub struct UblClient {
    endpoint: String,
    container_id: String,
    client: Client,
    timeout: Duration,
    signing_key: SigningKey,
    pubkey_hex: String,
}

impl UblClient {
    /// Create a new UBL client with signing key
    pub fn new(endpoint: &str, container_id: &str, timeout_ms: u64, signing_key: SigningKey) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .unwrap_or_else(|e| {
                tracing::error!("Failed to create UBL HTTP client: {}", e);
                std::process::exit(1);
            });

        let pubkey_hex = hex::encode(signing_key.verifying_key().as_bytes());

        Self {
            endpoint: endpoint.to_string(),
            container_id: container_id.to_string(),
            client,
            timeout: Duration::from_millis(timeout_ms),
            signing_key,
            pubkey_hex,
        }
    }

    /// Create with a generated keypair (for testing/development)
    pub fn with_generated_key(endpoint: &str, container_id: &str, timeout_ms: u64) -> Self {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        Self::new(endpoint, container_id, timeout_ms, signing_key)
    }

    /// Get the public key hex
    pub fn pubkey_hex(&self) -> &str {
        &self.pubkey_hex
    }

    /// Sign data with the client's key
    pub fn sign(&self, data: &[u8]) -> String {
        use ed25519_dalek::Signer;
        let signature = self.signing_key.sign(data);
        hex::encode(signature.to_bytes())
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.endpoint);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get ledger state for an entity
    pub async fn get_state(&self, entity_id: &str) -> Result<LedgerState> {
        let url = format!("{}/state/{}", self.endpoint, entity_id);

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            // Return default state for new entities
            return Ok(LedgerState::default());
        }

        resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))
    }

    /// Get recent events for an entity from C.Office audit log projection
    /// NOTE: Uses the new /query/office/audit endpoint instead of /ledger/:id/events
    pub async fn get_events(&self, entity_id: &EntityId, limit: usize) -> Result<Vec<LedgerEvent>> {
        // Use the C.Office audit log projection
        let url = format!(
            "{}/query/office/audit?entity_id={}&limit={}",
            self.endpoint, entity_id, limit
        );

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        // Parse the response and convert to LedgerEvent format
        #[derive(serde::Deserialize)]
        struct AuditResponse {
            ok: bool,
            data: Vec<AuditRow>,
        }
        
        #[derive(serde::Deserialize)]
        struct AuditRow {
            event_type: String,
            event_data: serde_json::Value,
            created_at_ms: i64,
            entry_hash: Option<String>,
            sequence: Option<u64>,
            author_pubkey: Option<String>,
        }

        let audit_resp: AuditResponse = resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))?;

        // Convert audit rows to LedgerEvents
        let events = audit_resp.data.into_iter().map(|row| {
            LedgerEvent {
                entry_hash: row.entry_hash.unwrap_or_default(),
                sequence: row.sequence.unwrap_or(0),
                author_pubkey: row.author_pubkey.unwrap_or_default(),
                intent_class: row.event_type.clone(),
                timestamp: chrono::DateTime::from_timestamp_millis(row.created_at_ms)
                    .unwrap_or_else(chrono::Utc::now),
                summary: row.event_type,
                data: row.event_data,
            }
        }).collect();

        Ok(events)
    }

    /// Get events after a specific timestamp
    pub async fn get_events_after(
        &self,
        entity_id: &EntityId,
        after: DateTime<Utc>,
    ) -> Result<Vec<LedgerEvent>> {
        let url = format!(
            "{}/ledger/{}/events?after={}",
            self.endpoint, entity_id, after.timestamp()
        );

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))
    }

    /// Get available affordances for an entity
    /// NOTE: Affordances are derived from available actions in C.Jobs
    /// For now returns a static list - in production, query C.Jobs/policy
    pub async fn get_affordances(&self, entity_id: &EntityId) -> Result<Vec<UblAffordance>> {
        // Static affordances for now - in production these would come from
        // the entity's policy and available tools
        Ok(vec![
            UblAffordance {
                id: "chat_reply".to_string(),
                name: "Reply in Chat".to_string(),
                description: "Send a message in the conversation".to_string(),
                risk_score: 0.1,
                parameters: None,
            },
            UblAffordance {
                id: "propose_job".to_string(),
                name: "Propose a Job".to_string(),
                description: "Create a job card for user approval".to_string(),
                risk_score: 0.3,
                parameters: None,
            },
            UblAffordance {
                id: "execute_tool".to_string(),
                name: "Execute Tool".to_string(),
                description: "Call an approved tool with parameters".to_string(),
                risk_score: 0.5,
                parameters: None,
            },
            UblAffordance {
                id: "escalate".to_string(),
                name: "Escalate to Guardian".to_string(),
                description: "Request human oversight for complex decision".to_string(),
                risk_score: 0.0,
                parameters: None,
            },
        ])
    }

    /// Get pending obligations for an entity
    /// NOTE: Obligations are derived from pending jobs in C.Jobs
    pub async fn get_obligations(&self, entity_id: &EntityId) -> Result<Vec<UblObligation>> {
        // Query pending jobs assigned to this entity
        let url = format!("{}/query/jobs?assigned_to={}&status=pending", self.endpoint, entity_id);

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        #[derive(serde::Deserialize)]
        struct JobsResponse {
            ok: bool,
            data: Vec<JobRow>,
        }
        
        #[derive(serde::Deserialize)]
        struct JobRow {
            job_id: String,
            title: String,
            description: Option<String>,
            priority: Option<String>,
            created_at: Option<i64>,
        }

        let jobs_resp: JobsResponse = resp.json().await.unwrap_or(JobsResponse { ok: false, data: vec![] });

        // Convert jobs to obligations
        let obligations = jobs_resp.data.into_iter().map(|job| {
            UblObligation {
                id: job.job_id,
                description: job.description.unwrap_or(job.title),
                due_at: None,
                priority: match job.priority.as_deref() {
                    Some("high") => 9,
                    Some("medium") => 5,
                    _ => 3,
                },
                source: "C.Jobs".to_string(),
            }
        }).collect();

        Ok(obligations)
    }

    /// Get the last handover for an entity
    /// NOTE: Uses /query/office/entities/:id/handovers/latest projection
    pub async fn get_last_handover(&self, entity_id: &EntityId) -> Result<Option<String>> {
        let url = format!("{}/query/office/entities/{}/handovers/latest", self.endpoint, entity_id);

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        // Parse the projection response format
        #[derive(serde::Deserialize)]
        struct ProjectionResponse {
            ok: bool,
            data: Option<HandoverRow>,
        }
        
        #[derive(serde::Deserialize)]
        struct HandoverRow {
            content: serde_json::Value,
        }

        let resp_data: ProjectionResponse = resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))?;

        // Extract content from handover JSON
        if let Some(handover) = resp_data.data {
            // Try to extract "summary" from content, or stringify the whole thing
            let content_str = handover.content
                .get("summary")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| serde_json::to_string(&handover.content).unwrap_or_default());
            Ok(Some(content_str))
        } else {
            Ok(None)
        }
    }

    /// Get handovers for an entity
    pub async fn get_handovers(&self, entity_id: &EntityId, limit: usize) -> Result<Vec<Handover>> {
        let url = format!(
            "{}/entities/{}/handovers?limit={}",
            self.endpoint, entity_id, limit
        );

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))
    }

    /// Get guardian info
    pub async fn get_guardian(&self, guardian_id: &str) -> Result<GuardianResponse> {
        let url = format!("{}/guardians/{}", self.endpoint, guardian_id);

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(OfficeError::UblError("Guardian not found".to_string()));
        }

        resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))
    }

    /// Get resolved issues
    pub async fn get_resolved_issues(&self, entity_id: &EntityId) -> Result<Vec<ResolvedIssue>> {
        let url = format!("{}/entities/{}/issues?status=resolved", self.endpoint, entity_id);

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))
    }

    /// Get trajectories (session history)
    pub async fn get_trajectories(&self, entity_id: &EntityId, days: u32) -> Result<Vec<Trajectory>> {
        let url = format!(
            "{}/entities/{}/trajectories?days={}",
            self.endpoint, entity_id, days
        );

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))
    }

    /// Commit a link to the ledger
    pub async fn commit(&self, link: LinkCommit) -> Result<CommitResponse> {
        let url = format!("{}/link/commit", self.endpoint);

        let resp = self.client.post(&url)
            .json(&link)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Request failed: {}", e)))?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(OfficeError::UblError(format!("Commit failed: {}", error_text)));
        }

        resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Parse failed: {}", e)))
    }

    /// Subscribe to event stream (SSE)
    pub async fn subscribe(&self, entity_id: &EntityId) -> Result<EventStream> {
        let url = format!("{}/ledger/{}/tail", self.endpoint, entity_id);
        EventStream::connect(&url).await
    }

    /// Commit an atom with proper signing
    pub async fn commit_atom(
        &self,
        container_id: &str,
        atom: &serde_json::Value,
        intent_class: &str,
        physics_delta: i64,
    ) -> Result<CommitResponse> {
        // Get current state to build the link
        let state = self.get_state(container_id).await?;

        // Canonicalize the atom
        let canonical = ubl_atom::canonicalize(atom)
            .map_err(|e| OfficeError::UblError(format!("Canonicalize failed: {}", e)))?;
        
        // Hash the atom (no domain tag per JSON✯Atomic binding)
        let atom_hash = ubl_kernel::hash_atom(&canonical);

        // Build link data for signing (without signature)
        let link_data = serde_json::json!({
            "version": 1,
            "container_id": container_id,
            "expected_sequence": state.sequence + 1,
            "previous_hash": state.last_hash,
            "atom_hash": atom_hash,
            "intent_class": intent_class,
            "physics_delta": physics_delta,
            "pact": null,
        });

        // Canonicalize link for signing
        let link_canonical = ubl_atom::canonicalize(&link_data)
            .map_err(|e| OfficeError::UblError(format!("Link canonicalize failed: {}", e)))?;

        // Sign with Ed25519
        let signature = self.sign(&link_canonical);

        // Build final link commit
        let link = LinkCommit {
            version: 1,
            container_id: container_id.to_string(),
            expected_sequence: state.sequence + 1,
            previous_hash: state.last_hash,
            atom_hash,
            intent_class: intent_class.to_string(),
            physics_delta,
            pact: None,
            author_pubkey: self.pubkey_hex.clone(),
            signature,
        };

        self.commit(link).await
    }

    /// Commit a pre-built link (caller is responsible for signing)
    pub async fn commit_signed(&self, link: LinkCommit) -> Result<CommitResponse> {
        self.commit(link).await
    }

    /// Request a permit from UBL (v1.1 endpoint)
    ///
    /// This is the canonical way to authorize mutations.
    /// Office MUST call this before any mutation.
    pub async fn request_permit(
        &self,
        request: &crate::middleware::PermitRequest,
    ) -> Result<crate::middleware::PermitResponse> {
        let url = format!("{}/v1/policy/permit", self.endpoint);

        let resp = self.client.post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Permit request failed: {}", e)))?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            // Try to parse as denial response
            if let Ok(denial) = serde_json::from_str::<crate::middleware::PermitResponse>(&error_text) {
                return Ok(denial);
            }
            return Err(OfficeError::UblError(format!("Permit request failed: {}", error_text)));
        }

        resp.json().await
            .map_err(|e| OfficeError::UblError(format!("Permit parse failed: {}", e)))
    }

    /// Issue a command to the UBL (v1.1 endpoint)
    pub async fn issue_command(&self, command: &CommandEnvelope) -> Result<()> {
        let url = format!("{}/v1/commands/issue", self.endpoint);

        let resp = self.client.post(&url)
            .json(&command)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Command issue failed: {}", e)))?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(OfficeError::UblError(format!("Command issue failed: {}", error_text)));
        }

        Ok(())
    }

    /// Submit execution receipt to UBL (v1.1 endpoint)
    pub async fn submit_receipt(&self, receipt: &ExecutionReceipt) -> Result<()> {
        let url = format!("{}/v1/exec.finish", self.endpoint);

        let resp = self.client.post(&url)
            .json(&receipt)
            .send()
            .await
            .map_err(|e| OfficeError::UblError(format!("Receipt submit failed: {}", e)))?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(OfficeError::UblError(format!("Receipt submit failed: {}", error_text)));
        }

        Ok(())
    }

    /// Check if UBL endpoint is healthy
    pub async fn is_healthy(&self, entity_id: &str) -> bool {
        self.health().await.unwrap_or(false)
    }

    /// Build and sign a link commit for an event
    pub async fn publish_event(
        &self,
        container_id: &str,
        event: &serde_json::Value,
    ) -> Result<CommitResponse> {
        self.commit_atom(container_id, event, "observation", 0).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HandoverResponse {
    content: String,
    session_id: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianResponse {
    pub id: String,
    pub name: String,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedIssue {
    pub id: String,
    pub description: String,
    pub resolved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub session_id: String,
    pub session_type: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub tokens_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCommit {
    pub version: u8,
    pub container_id: String,
    pub expected_sequence: u64,
    pub previous_hash: String,
    pub atom_hash: String,
    pub intent_class: String,
    pub physics_delta: i64,
    pub pact: Option<PactProof>,
    pub author_pubkey: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactProof {
    pub pact_id: String,
    pub signatures: Vec<PactSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactSignature {
    pub pubkey: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResponse {
    pub ok: bool,
    pub entry_hash: String,
    pub sequence: u64,
}

/// Command envelope for v1.1 protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub jti: String,
    pub tenant_id: String,
    #[serde(rename = "jobId")]
    pub job_id: String,
    #[serde(rename = "jobType")]
    pub job_type: String,
    pub params: serde_json::Value,
    pub subject_hash: String,
    pub policy_hash: String,
    pub permit: serde_json::Value,
    pub target: String,
    pub office_id: String,
}

/// Execution receipt for v1.1 protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub tenant_id: String,
    #[serde(rename = "jobId")]
    pub job_id: String,
    pub status: String,
    pub finished_at: u64,
    pub logs_hash: String,
    pub artifacts: Vec<String>,
    pub usage: serde_json::Value,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = UblClient::with_generated_key("http://localhost:3000", "office", 30000);
        assert_eq!(client.endpoint, "http://localhost:3000");
        assert_eq!(client.container_id, "office");
        assert!(!client.pubkey_hex().is_empty());
        assert_eq!(client.pubkey_hex().len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_signing() {
        let client = UblClient::with_generated_key("http://localhost:3000", "office", 30000);
        let data = b"test data to sign";
        let signature = client.sign(data);
        assert_eq!(signature.len(), 128); // 64 bytes = 128 hex chars
    }
}
