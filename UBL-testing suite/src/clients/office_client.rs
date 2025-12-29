use anyhow::Result;
use reqwest::Client;
use serde: :{Deserialize, Serialize};
use serde_json::Value;

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
    
    pub async fn health(&self) -> Result<HealthResponse> {
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
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
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