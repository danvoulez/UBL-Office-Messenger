//! Permit Middleware - UBL Sovereignty Enforcement
//!
//! From ADR-UBL-Console-001 v1.1:
//! > Toda execução mutável exige Permit emitido pelo UBL
//!
//! This middleware ensures:
//! 1. Every mutation calls /v1/policy/permit on UBL
//! 2. Only Allow responses proceed
//! 3. Deny = fail-closed (no execution)
//!
//! "Office não pode pular o Permit nem registrar recibos fora do UBL."

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ubl_client::UblClient;

/// Errors from permit middleware
#[derive(Error, Debug)]
pub enum PermitError {
    #[error("Permit denied by UBL: {reason}")]
    Denied { reason: String },

    #[error("Permit request failed: {0}")]
    RequestFailed(String),

    #[error("Missing required binding: {0}")]
    MissingBinding(String),

    #[error("Invalid permit response: {0}")]
    InvalidResponse(String),

    #[error("Permit expired")]
    Expired,
}

/// Request for a permit from UBL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermitRequest {
    /// Tenant ID (required)
    pub tenant_id: String,
    /// Actor ID (who is requesting)
    pub actor_id: String,
    /// Intent description
    pub intent: String,
    /// Context for policy evaluation
    pub context: serde_json::Value,
    /// Job type (from allowlist)
    pub job_type: String,
    /// Job parameters
    pub params: serde_json::Value,
    /// Target (LAB_512, LAB_256, LAB_8GB)
    pub target: String,
    /// Approval reference (for L3+ actions)
    pub approval_ref: Option<String>,
}

/// Response from UBL permit endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermitResponse {
    /// The permit (if allowed)
    pub permit: Option<Permit>,
    /// Policy hash used for evaluation
    pub policy_hash: String,
    /// Subject hash (hash of params)
    pub subject_hash: String,
    /// Whether permit was granted
    pub allowed: bool,
    /// Denial reason (if denied)
    pub denial_reason: Option<String>,
}

/// A permit from UBL (v1.1 format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permit {
    /// Audience (runner target)
    pub aud: String,
    /// JWT ID (unique, single-use)
    pub jti: String,
    /// Expiration timestamp (ms)
    pub exp: u64,
    /// Signature
    pub sig: String,
    /// Scopes
    pub scopes: PermitScopes,
}

/// Scopes within a permit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermitScopes {
    pub tenant_id: String,
    #[serde(rename = "jobType")]
    pub job_type: String,
    pub target: String,
    pub subject_hash: String,
    pub policy_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_ref: Option<String>,
}

/// Permit middleware - enforces UBL sovereignty
pub struct PermitMiddleware {
    ubl_client: Arc<UblClient>,
    /// Required bindings (from office.constitution.yaml)
    required_bindings: Vec<String>,
}

impl PermitMiddleware {
    /// Create new permit middleware
    pub fn new(ubl_client: Arc<UblClient>) -> Self {
        Self {
            ubl_client,
            required_bindings: vec![
                "permit.scopes.tenant_id".to_string(),
                "permit.scopes.jobType".to_string(),
                "permit.scopes.target".to_string(),
                "permit.scopes.subject_hash".to_string(),
                "permit.scopes.policy_hash".to_string(),
            ],
        }
    }

    /// Request a permit from UBL
    ///
    /// This is the ONLY way to authorize a mutation in Office.
    /// If UBL denies, Office MUST NOT proceed.
    pub async fn request_permit(&self, request: PermitRequest) -> Result<PermitResponse, PermitError> {
        // Validate required fields
        self.validate_request(&request)?;

        // Call UBL /v1/policy/permit
        let response = self.ubl_client
            .request_permit(&request)
            .await
            .map_err(|e| PermitError::RequestFailed(e.to_string()))?;

        // Check if allowed
        if !response.allowed {
            return Err(PermitError::Denied {
                reason: response.denial_reason.unwrap_or_else(|| "Unknown reason".to_string()),
            });
        }

        // Validate permit has all required bindings
        if let Some(ref permit) = response.permit {
            self.validate_permit(permit)?;
        } else {
            return Err(PermitError::InvalidResponse("Permit missing in Allow response".to_string()));
        }

        Ok(response)
    }

    /// Validate permit request has required fields
    fn validate_request(&self, request: &PermitRequest) -> Result<(), PermitError> {
        if request.tenant_id.is_empty() {
            return Err(PermitError::MissingBinding("tenant_id".to_string()));
        }
        if request.job_type.is_empty() {
            return Err(PermitError::MissingBinding("job_type".to_string()));
        }
        if request.target.is_empty() {
            return Err(PermitError::MissingBinding("target".to_string()));
        }
        Ok(())
    }

    /// Validate permit has all required bindings
    fn validate_permit(&self, permit: &Permit) -> Result<(), PermitError> {
        // Check expiration
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if permit.exp < now_ms {
            return Err(PermitError::Expired);
        }

        // Check required scopes
        if permit.scopes.tenant_id.is_empty() {
            return Err(PermitError::MissingBinding("permit.scopes.tenant_id".to_string()));
        }
        if permit.scopes.job_type.is_empty() {
            return Err(PermitError::MissingBinding("permit.scopes.jobType".to_string()));
        }
        if permit.scopes.target.is_empty() {
            return Err(PermitError::MissingBinding("permit.scopes.target".to_string()));
        }
        if permit.scopes.subject_hash.is_empty() {
            return Err(PermitError::MissingBinding("permit.scopes.subject_hash".to_string()));
        }
        if permit.scopes.policy_hash.is_empty() {
            return Err(PermitError::MissingBinding("permit.scopes.policy_hash".to_string()));
        }

        Ok(())
    }

    /// Check if a permit is still valid
    pub fn is_permit_valid(&self, permit: &Permit) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        permit.exp > now_ms
    }
}

/// Helper to build permit requests
pub struct PermitRequestBuilder {
    request: PermitRequest,
}

impl PermitRequestBuilder {
    pub fn new(tenant_id: &str, actor_id: &str, job_type: &str) -> Self {
        Self {
            request: PermitRequest {
                tenant_id: tenant_id.to_string(),
                actor_id: actor_id.to_string(),
                intent: String::new(),
                context: serde_json::json!({}),
                job_type: job_type.to_string(),
                params: serde_json::json!({}),
                target: String::new(),
                approval_ref: None,
            },
        }
    }

    pub fn intent(mut self, intent: &str) -> Self {
        self.request.intent = intent.to_string();
        self
    }

    pub fn context(mut self, context: serde_json::Value) -> Self {
        self.request.context = context;
        self
    }

    pub fn params(mut self, params: serde_json::Value) -> Self {
        self.request.params = params;
        self
    }

    pub fn target(mut self, target: &str) -> Self {
        self.request.target = target.to_string();
        self
    }

    pub fn approval_ref(mut self, approval_ref: &str) -> Self {
        self.request.approval_ref = Some(approval_ref.to_string());
        self
    }

    pub fn build(self) -> PermitRequest {
        self.request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permit_request_builder() {
        let request = PermitRequestBuilder::new("T.UBL", "U.dan", "service.restart")
            .intent("Restart minio service")
            .target("LAB_512")
            .params(serde_json::json!({"name": "minio"}))
            .build();

        assert_eq!(request.tenant_id, "T.UBL");
        assert_eq!(request.actor_id, "U.dan");
        assert_eq!(request.job_type, "service.restart");
        assert_eq!(request.target, "LAB_512");
    }

    #[test]
    fn test_permit_validation() {
        let permit = Permit {
            aud: "runner:LAB_512".to_string(),
            jti: "test-jti".to_string(),
            exp: u64::MAX, // Far future
            sig: "test-sig".to_string(),
            scopes: PermitScopes {
                tenant_id: "T.UBL".to_string(),
                job_type: "service.restart".to_string(),
                target: "LAB_512".to_string(),
                subject_hash: "abc123".to_string(),
                policy_hash: "def456".to_string(),
                approval_ref: None,
            },
        };

        // Would need UblClient mock for full test
        // Just verify struct works
        assert_eq!(permit.scopes.tenant_id, "T.UBL");
    }
}

