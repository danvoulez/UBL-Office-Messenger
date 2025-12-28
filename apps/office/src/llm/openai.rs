//! OpenAI GPT Provider

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, LlmUsage, MessageRole};
use crate::{OfficeError, Result};

/// OpenAI GPT provider
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    client: Client,
}

impl OpenAIProvider {
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
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    choices: Vec<OpenAIChoice>,
    model: String,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn chat(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Convert messages
        let mut messages: Vec<OpenAIMessage> = Vec::new();

        // Add system message if provided
        if let Some(system) = request.system {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: system,
            });
        }

        // Add conversation messages
        for m in request.messages {
            let role = match m.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            };
            messages.push(OpenAIMessage {
                role: role.to_string(),
                content: m.content,
            });
        }

        let openai_request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            max_tokens: request.max_tokens.min(self.max_tokens),
            temperature: request.temperature.min(self.temperature.max(0.0)),
            stop: request.stop_sequences,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| OfficeError::LlmError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OfficeError::LlmError(format!("API error: {}", error_text)));
        }

        let openai_response: OpenAIResponse = response.json().await
            .map_err(|e| OfficeError::LlmError(format!("Parse failed: {}", e)))?;

        let choice = openai_response.choices.into_iter().next()
            .ok_or_else(|| OfficeError::LlmError("No choices returned".to_string()))?;

        Ok(LlmResponse {
            content: choice.message.content,
            finish_reason: choice.finish_reason.unwrap_or_else(|| "unknown".to_string()),
            usage: LlmUsage {
                input_tokens: openai_response.usage.prompt_tokens,
                output_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
            model: openai_response.model,
        })
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}
