use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone)]
pub struct UblClient {
    base_url: String,
    client: Client,
}

impl UblClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }
    
    pub async fn health(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }
    
    pub async fn bootstrap(&self, tenant_id: &str) -> Result<BootstrapResponse> {
        let url = format!("{}/messenger/bootstrap?tenant_id={}", self.base_url, tenant_id);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }
    
    pub async fn send_message(&self, req: SendMessageRequest) -> Result<SendMessageResponse> {
        let url = format!("{}/v1/conversations/{}/messages", self.base_url, req.conversation_id);
        let resp = self.client
            .post(&url)
            .json(&req)
            .send()
            .await?;
        Ok(resp.json().await?)
    }
    
    pub async fn approve_job(&self, job_id: &str, req: JobActionRequest) -> Result<JobActionResponse> {
        let url = format!("{}/v1/jobs/{}/actions", self.base_url, job_id);
        let resp = self.client
            .post(&url)
            .json(&req)
            .send()
            .await?;
        Ok(resp.json().await?)
    }
    
    pub async fn get_conversation_timeline(
        &self,
        conversation_id: &str,
        cursor: Option<&str>,
    ) -> Result<TimelineResponse> {
        let mut url = format!("{}/v1/conversations/{}/timeline", self.base_url, conversation_id);
        if let Some(c) = cursor {
            url.push_str(&format!("?cursor={}", c));
        }
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }
    
    pub async fn get_job(&self, job_id: &str) -> Result<JobResponse> {
        let url = format!("{}/v1/jobs/{}", self.base_url, job_id);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }
    
    pub async fn subscribe_to_stream(&self, tenant_id: &str) -> Result<eventsource_client::Client> {
        let url = format!("{}/v1/stream?tenant_id={}", self.base_url, tenant_id);
        Ok(eventsource_client::ClientBuilder::for_url(&url)?
            .build())
    }
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct BootstrapResponse {
    pub entities: Vec<Value>,
    pub conversations: Vec<Value>,
    pub messages: Vec<Value>,
}

#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub conversation_id: String, // Used in URL path
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: String,
    pub hash: String,
    pub sequence: i64,
    pub action: String, // "committed" | "office_processing"
}

#[derive(Debug, Serialize)]
pub struct JobActionRequest {
    pub action_type: String,
    pub card_id: String,
    pub button_id: String,
    pub input_data: Option<Value>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JobActionResponse {
    pub success: bool,
    pub event_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TimelineResponse {
    pub items: Vec<Value>,
    pub cursor: String,
}

#[derive(Debug, Deserialize)]
pub struct JobResponse {
    pub job_id: String,
    pub title: String,
    pub state: String,
    pub timeline: Vec<Value>,
    pub artifacts: Vec<Value>,
}