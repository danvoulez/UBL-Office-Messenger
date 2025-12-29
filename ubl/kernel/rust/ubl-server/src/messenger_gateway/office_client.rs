//! Office HTTP Client
//!
//! HTTP client for communicating with Office runtime.
//! Used by Gateway to forward messages and job actions.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

/// Office client configuration
pub struct OfficeClient {
    base_url: String,
    client: reqwest::Client,
}

impl OfficeClient {
    /// Create a new Office client
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { base_url, client }
    }

    /// Ingest a message from Gateway
    /// Office decides: reply or propose job
    pub async fn ingest_message(
        &self,
        req: &IngestMessageRequest,
    ) -> Result<IngestMessageResponse, OfficeClientError> {
        let url = format!("{}/v1/office/ingest_message", self.base_url);
        
        info!("📨 Gateway → Office: ingest_message conversation={} message={}", 
              req.conversation_id, req.message_id);
        
        let response = self.client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| OfficeClientError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Office ingest_message failed: {}", error_text);
            return Err(OfficeClientError::Office(error_text));
        }

        let result: IngestMessageResponse = response
            .json()
            .await
            .map_err(|e| OfficeClientError::Parse(e.to_string()))?;

        info!("✅ Office response: action={:?} job_id={:?}", 
              result.action, result.job_id);
        
        Ok(result)
    }

    /// Handle a job action (approve/reject/provide_input)
    pub async fn job_action(
        &self,
        req: &JobActionRequest,
    ) -> Result<JobActionResponse, OfficeClientError> {
        let url = format!("{}/v1/office/job_action", self.base_url);
        
        info!("🔧 Gateway → Office: job_action job={} action={}", 
              req.job_id, req.action_type);
        
        let response = self.client
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| OfficeClientError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Office job_action failed: {}", error_text);
            return Err(OfficeClientError::Office(error_text));
        }

        let result: JobActionResponse = response
            .json()
            .await
            .map_err(|e| OfficeClientError::Parse(e.to_string()))?;

        info!("✅ Office job_action response: success={}", result.success);
        
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestMessageRequest {
    pub conversation_id: String,
    pub message_id: String,
    pub from: String,
    pub content: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestMessageResponse {
    pub action: MessageAction,
    pub reply_content: Option<String>,
    pub job_id: Option<String>,
    pub card: Option<serde_json::Value>, // JobCard JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageAction {
    Reply,
    ProposeJob,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobActionRequest {
    pub job_id: String,
    pub action_type: String, // approve, reject, provide_input, etc.
    pub button_id: String,
    pub card_id: String,
    pub input_data: Option<serde_json::Value>,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobActionResponse {
    pub success: bool,
    pub updated_card: Option<serde_json::Value>,
    pub event_ids: Vec<String>,
}

#[derive(Debug)]
pub enum OfficeClientError {
    Network(String),
    Office(String),
    Parse(String),
}

impl std::fmt::Display for OfficeClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfficeClientError::Network(e) => write!(f, "Network error: {}", e),
            OfficeClientError::Office(e) => write!(f, "Office error: {}", e),
            OfficeClientError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for OfficeClientError {}

