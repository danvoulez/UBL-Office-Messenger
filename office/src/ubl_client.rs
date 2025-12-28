//! UBL Client for Office
//!
//! HTTP-only client that speaks to the UBL Gateway.
//! Uses ASC (Agent Signing Certificate) for authentication.
//! NEVER touches the database directly.

use crate::middleware::constitution;
use anyhow::{anyhow, Result};
use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error, info};

/// UBL Client - Office's only interface to the UBL world
pub struct UblClient {
    /// Base URL of the UBL Gateway (e.g., http://10.77.0.1:8080)
    base: String,
    /// ASC token for authentication
    asc: String,
    /// HTTP client
    client: Client,
}

impl UblClient {
    /// Create a new UBL client
    pub fn new(base: String, asc: String) -> Self {
        Self {
            base,
            asc,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Make a GET request to the UBL Gateway
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        
        // Constitution check
        constitution::validate_outbound_url(&url)?;
        
        debug!("GET {}", url);
        
        let res = self.client
            .get(&url)
            .header("Authorization", format!("ASC {}", self.asc))
            .send()
            .await?;
        
        self.handle_response(res).await
    }

    /// Make a POST request to the UBL Gateway
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        
        // Constitution check
        constitution::validate_outbound_url(&url)?;
        
        debug!("POST {}", url);
        
        let res = self.client
            .post(&url)
            .header("Authorization", format!("ASC {}", self.asc))
            .json(body)
            .send()
            .await?;
        
        self.handle_response(res).await
    }

    /// Make a POST request with JSON value
    pub async fn post_json<T: DeserializeOwned>(&self, path: &str, body: &serde_json::Value) -> Result<T> {
        self.post(path, body).await
    }

    /// Health check
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base);
        constitution::validate_outbound_url(&url)?;
        
        let res = self.client.get(&url).send().await?;
        Ok(res.status().is_success())
    }

    /// Request a permit for an operation
    pub async fn request_permit(&self, request: &PermitRequest) -> Result<PermitResponse> {
        self.post("/v1/policy/permit", request).await
    }

    /// Issue a command for Runner to execute
    pub async fn issue_command(&self, command: &CommandEnvelope) -> Result<()> {
        let _: serde_json::Value = self.post("/v1/commands/issue", command).await?;
        Ok(())
    }

    /// Submit execution receipt
    pub async fn submit_receipt(&self, receipt: &Receipt) -> Result<()> {
        let _: serde_json::Value = self.post("/v1/exec.finish", receipt).await?;
        Ok(())
    }

    /// Handle HTTP response
    async fn handle_response<T: DeserializeOwned>(&self, res: Response) -> Result<T> {
        let status = res.status();
        
        if !status.is_success() {
            let text = res.text().await.unwrap_or_default();
            error!("UBL API error {}: {}", status, text);
            return Err(anyhow!("UBL API error {}: {}", status, text));
        }
        
        Ok(res.json::<T>().await?)
    }
}

// ============ Request/Response Types ============

#[derive(Debug, Serialize)]
pub struct PermitRequest {
    pub tenant_id: String,
    pub actor_id: String,
    pub intent: String,
    pub context: serde_json::Value,
    #[serde(rename = "jobType")]
    pub job_type: String,
    pub params: serde_json::Value,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_ref: Option<String>,
    pub risk: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webauthn_assertion: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PermitResponse {
    pub permit: serde_json::Value,
    pub policy_hash: String,
    pub subject_hash: String,
    pub allowed: bool,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct Receipt {
    pub tenant_id: String,
    #[serde(rename = "jobId")]
    pub job_id: String,
    pub status: String,
    pub finished_at: u64,
    pub logs_hash: String,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub usage: serde_json::Value,
    #[serde(default)]
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constitution_enforcement() {
        // This should pass
        assert!(constitution::validate_outbound_url("http://10.77.0.1:8080/health").is_ok());
        
        // This should fail - external URL
        assert!(constitution::validate_outbound_url("https://api.openai.com").is_err());
    }
}

