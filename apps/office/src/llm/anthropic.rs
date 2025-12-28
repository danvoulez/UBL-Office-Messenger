//! Anthropic Claude Provider

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, LlmUsage, MessageRole};
use crate::{OfficeError, Result};

/// Anthropic Claude provider
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: &str, model: &str, max_tokens: u32, temperature: f32) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            max_tokens,
            temperature,
            client: Client::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    temperature: f32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop_sequences: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn chat(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Convert messages
        let mut system = request.system;
        let messages: Vec<AnthropicMessage> = request.messages
            .into_iter()
            .filter_map(|m| {
                match m.role {
                    MessageRole::System => {
                        system = Some(m.content);
                        None
                    }
                    MessageRole::User => Some(AnthropicMessage {
                        role: "user".to_string(),
                        content: m.content,
                    }),
                    MessageRole::Assistant => Some(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: m.content,
                    }),
                }
            })
            .collect();

        let anthropic_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens.min(self.max_tokens),
            messages,
            system,
            temperature: request.temperature.min(self.temperature.max(0.0)),
            stop_sequences: request.stop_sequences,
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| OfficeError::LlmError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OfficeError::LlmError(format!("API error: {}", error_text)));
        }

        let anthropic_response: AnthropicResponse = response.json().await
            .map_err(|e| OfficeError::LlmError(format!("Parse failed: {}", e)))?;

        let content = anthropic_response.content
            .into_iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(LlmResponse {
            content,
            finish_reason: anthropic_response.stop_reason.unwrap_or_else(|| "unknown".to_string()),
            usage: LlmUsage {
                input_tokens: anthropic_response.usage.input_tokens,
                output_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
            },
            model: anthropic_response.model,
        })
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}
