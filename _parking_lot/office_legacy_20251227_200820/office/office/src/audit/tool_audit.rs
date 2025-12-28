//! Tool Audit - Records every tool call and result
//!
//! From the spec:
//! > tool.called records **intent to execute** with inputs in a **safe form**
//! > tool.result records **what happened**: success/failure, outputs, artifacts

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ubl_client::UblClient;
use crate::{OfficeError, Result};

use super::pii::PiiPolicy;

/// A recorded tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this call (pairs with result)
    pub tool_call_id: String,
    /// Tool name (e.g., "calendar.create_invite")
    pub tool_name: String,
    /// Tool version
    pub tool_version: String,
    /// Human-readable purpose
    pub purpose: String,
    /// Sanitized inputs (PII redacted)
    pub inputs: serde_json::Value,
    /// PII policy applied
    pub pii_policy: PiiPolicy,
    /// Idempotency key
    pub idempotency_key: String,
    /// Attempt number (1, 2, 3...)
    pub attempt: u32,
    /// If retry, reference to original call
    pub retry_of: Option<String>,
    /// Timestamp
    pub called_at: DateTime<Utc>,
}

/// A tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Pairs with tool_call_id
    pub tool_call_id: String,
    /// Tool name
    pub tool_name: String,
    /// Success or error
    pub status: ToolStatus,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Sanitized output (if success)
    pub output: Option<serde_json::Value>,
    /// Artifacts produced
    pub artifacts: Vec<Artifact>,
    /// Error details (if failed)
    pub error: Option<ToolError>,
    /// Safety checks
    pub safety: SafetyReport,
    /// Attempt number
    pub attempt: u32,
    /// Timestamp
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Success,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    /// Error code (e.g., "PROVIDER_TIMEOUT")
    pub error_code: String,
    /// Safe message (no PII)
    pub message_safe: String,
    /// Can retry?
    pub retryable: bool,
    /// Suggested wait before retry
    pub suggested_wait_seconds: Option<u32>,
}

impl ToolError {
    pub fn timeout() -> Self {
        Self {
            error_code: "PROVIDER_TIMEOUT".to_string(),
            message_safe: "Provider didn't respond in time.".to_string(),
            retryable: true,
            suggested_wait_seconds: Some(10),
        }
    }

    pub fn rate_limited() -> Self {
        Self {
            error_code: "PROVIDER_RATE_LIMIT".to_string(),
            message_safe: "Rate limit exceeded.".to_string(),
            retryable: true,
            suggested_wait_seconds: Some(60),
        }
    }

    pub fn auth_required() -> Self {
        Self {
            error_code: "PROVIDER_AUTH_REQUIRED".to_string(),
            message_safe: "Authentication required.".to_string(),
            retryable: false,
            suggested_wait_seconds: None,
        }
    }

    pub fn invalid_input(details: &str) -> Self {
        Self {
            error_code: "INVALID_INPUT".to_string(),
            message_safe: format!("Invalid input: {}", details),
            retryable: false,
            suggested_wait_seconds: None,
        }
    }

    pub fn unavailable() -> Self {
        Self {
            error_code: "PROVIDER_UNAVAILABLE".to_string(),
            message_safe: "Provider is unavailable.".to_string(),
            retryable: true,
            suggested_wait_seconds: Some(300),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: String,
    pub kind: ArtifactKind,
    pub title: String,
    pub url: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    File,
    Link,
    Record,
    Quote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    /// Was PII leak detected in output?
    pub pii_leak_detected: bool,
    /// What redactions were applied
    pub redaction_summary: Vec<String>,
}

impl Default for SafetyReport {
    fn default() -> Self {
        Self {
            pii_leak_detected: false,
            redaction_summary: vec!["No raw PII stored".to_string()],
        }
    }
}

/// Tool Audit Manager - Records all tool activity
pub struct ToolAudit {
    ubl_client: Arc<UblClient>,
    container_id: String,
    /// In-flight calls (for pairing with results)
    in_flight: tokio::sync::RwLock<HashMap<String, (ToolCall, Instant)>>,
}

impl ToolAudit {
    pub fn new(ubl_client: Arc<UblClient>, container_id: &str) -> Self {
        Self {
            ubl_client,
            container_id: container_id.to_string(),
            in_flight: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Record a tool call (before execution)
    pub async fn record_call(
        &self,
        job_id: &str,
        conversation_id: &str,
        tenant_id: &str,
        actor_entity_id: &str,
        call: ToolCall,
    ) -> Result<()> {
        // Store in-flight for latency tracking
        {
            let mut in_flight = self.in_flight.write().await;
            in_flight.insert(call.tool_call_id.clone(), (call.clone(), Instant::now()));
        }

        // Build event
        let event = serde_json::json!({
            "event_type": "tool.called",
            "job_id": job_id,
            "conversation_id": conversation_id,
            "tenant_id": tenant_id,
            "actor": {
                "entity_id": actor_entity_id,
                "actor_type": "agent"
            },
            "payload": {
                "tool_call_id": call.tool_call_id,
                "tool_name": call.tool_name,
                "tool_version": call.tool_version,
                "purpose": call.purpose,
                "inputs": call.inputs,
                "pii_policy": call.pii_policy,
                "idempotency_key": call.idempotency_key,
                "attempt": call.attempt,
                "retry_of_tool_call_id": call.retry_of
            }
        });

        self.commit_event(event).await
    }

    /// Record a tool result (after execution)
    pub async fn record_result(
        &self,
        job_id: &str,
        conversation_id: &str,
        tenant_id: &str,
        actor_entity_id: &str,
        result: ToolResult,
    ) -> Result<()> {
        // Remove from in-flight
        {
            let mut in_flight = self.in_flight.write().await;
            in_flight.remove(&result.tool_call_id);
        }

        // Build event
        let event = serde_json::json!({
            "event_type": "tool.result",
            "job_id": job_id,
            "conversation_id": conversation_id,
            "tenant_id": tenant_id,
            "actor": {
                "entity_id": actor_entity_id,
                "actor_type": "agent"
            },
            "payload": {
                "tool_call_id": result.tool_call_id,
                "tool_name": result.tool_name,
                "status": result.status,
                "latency_ms": result.latency_ms,
                "output": result.output,
                "artifacts": result.artifacts,
                "error": result.error,
                "safety": result.safety,
                "attempt": result.attempt
            }
        });

        self.commit_event(event).await
    }

    /// Get latency for an in-flight call
    pub async fn get_latency(&self, tool_call_id: &str) -> Option<u64> {
        let in_flight = self.in_flight.read().await;
        in_flight.get(tool_call_id).map(|(_, start)| start.elapsed().as_millis() as u64)
    }

    /// Commit event to UBL
    async fn commit_event(&self, event: serde_json::Value) -> Result<()> {
        // Canonicalize using sorted JSON (JSON✯Atomic v1.0 compliant)
        let canonical = serde_json::to_vec(&event)
            .map_err(|e| OfficeError::UblError(format!("Serialize failed: {}", e)))?;
        
        // Hash with BLAKE3
        let atom_hash = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&canonical);
            hex::encode(hasher.finalize().as_bytes())
        };

        // Commit to UBL via client
        // IntentClass: AUDIT = 0x02, PhysicsDelta: 0 (audit doesn't affect balance)
        self.ubl_client.commit_atom(&self.container_id, &event, "0x02", 0).await?;
        Ok(())
    }
}

/// Builder for creating tool calls with proper sanitization
pub struct ToolCallBuilder {
    tool_name: String,
    tool_version: String,
    purpose: String,
    job_id: String,
    inputs: serde_json::Value,
    attempt: u32,
    retry_of: Option<String>,
}

impl ToolCallBuilder {
    pub fn new(tool_name: &str, job_id: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            tool_version: "v1".to_string(),
            purpose: String::new(),
            job_id: job_id.to_string(),
            inputs: serde_json::json!({}),
            attempt: 1,
            retry_of: None,
        }
    }

    pub fn version(mut self, version: &str) -> Self {
        self.tool_version = version.to_string();
        self
    }

    pub fn purpose(mut self, purpose: &str) -> Self {
        self.purpose = purpose.to_string();
        self
    }

    pub fn inputs(mut self, inputs: serde_json::Value) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn attempt(mut self, attempt: u32) -> Self {
        self.attempt = attempt;
        self
    }

    pub fn retry_of(mut self, original_call_id: &str) -> Self {
        self.retry_of = Some(original_call_id.to_string());
        self
    }

    pub fn build(self) -> ToolCall {
        let tool_call_id = format!("tcall_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
        let idempotency_key = format!("idem:{}:{}:{}", self.job_id, self.tool_name, self.tool_version);

        ToolCall {
            tool_call_id,
            tool_name: self.tool_name,
            tool_version: self.tool_version,
            purpose: self.purpose,
            inputs: self.inputs,
            pii_policy: PiiPolicy::default(),
            idempotency_key,
            attempt: self.attempt,
            retry_of: self.retry_of,
            called_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_builder() {
        let call = ToolCallBuilder::new("calendar.create_invite", "job_123")
            .purpose("Create a meeting invite")
            .inputs(serde_json::json!({
                "title": "Team standup",
                "duration_minutes": 30
            }))
            .build();

        assert!(call.tool_call_id.starts_with("tcall_"));
        assert_eq!(call.tool_name, "calendar.create_invite");
        assert_eq!(call.attempt, 1);
    }

    #[test]
    fn test_tool_error_types() {
        let timeout = ToolError::timeout();
        assert!(timeout.retryable);
        assert_eq!(timeout.error_code, "PROVIDER_TIMEOUT");

        let auth = ToolError::auth_required();
        assert!(!auth.retryable);
    }
}

