use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OfficeClient {
    base_url: String,
    client: Client,
}

impl OfficeClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }
    
    pub async fn health(&self) -> Result<OfficeHealthResponse> {
        let url = format!("{}/health", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }
    
    pub async fn create_entity(&self, req: CreateEntityRequest) -> Result<EntityResponse> {
        let url = format!("{}/entities", self.base_url);
        let resp = self.client
            .post(&url)
            .json(&req)
            .send()
            .await?;
        Ok(resp.json().await?)
    }
    
    pub async fn get_entity(&self, entity_id: &str) -> Result<EntityResponse> {
        let url = format!("{}/entities/{}", self.base_url, entity_id);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }
    
    pub async fn list_entities(&self) -> Result<Vec<EntityResponse>> {
        let url = format!("{}/entities", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }
    
    pub async fn execute_job(&self, req: ExecuteJobRequest) -> Result<ExecuteJobResponse> {
        let url = format!("{}/jobs/execute", self.base_url);
        let resp = self.client
            .post(&url)
            .json(&req)
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    // Prompt 2: Office ws/test endpoint
    pub async fn ws_test(&self, req: WsTestRequest, sid: Option<&str>, asc: Option<&str>) -> Result<WsTestResponse> {
        let url = format!("{}/office/ws/test", self.base_url);
        let mut request = self.client.post(&url).json(&req);
        
        if let Some(s) = sid {
            request = request.header("authorization", format!("Bearer {}", s));
        }
        if let Some(a) = asc {
            request = request.header("x-ubl-asc", a);
        }
        
        let resp = request.send().await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, text);
        }
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Deserialize)]
pub struct OfficeHealthResponse {
    pub status:  String,
}

#[derive(Debug, Serialize)]
pub struct CreateEntityRequest {
    pub name: String,
    pub entity_type: String,
}

#[derive(Debug, Deserialize)]
pub struct EntityResponse {
    pub entity_id: String,
    pub name: String,
    pub entity_type: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ExecuteJobRequest {
    pub job_id: String,
    pub entity_id: String,
    pub conversation_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteJobResponse {
    pub job_id: String,
    pub status: String,
}

// Prompt 2: Office ws/test types
#[derive(Debug, Serialize)]
pub struct WsTestRequest {
    pub tenant: String,
    pub workspace: String,
    pub repo: String,
    pub sha: String,
    pub suite: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct WsTestResponse {
    pub link_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<serde_json::Value>,
}