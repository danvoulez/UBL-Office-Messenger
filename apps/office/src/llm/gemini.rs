//! Google Gemini Provider

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, LlmUsage, MessageRole};
use crate::{OfficeError, Result};

/// Google Gemini provider
pub struct GeminiProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    client: Client,
}

impl GeminiProvider {
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

// Gemini API structures
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    temperature: f32,
    #[serde(rename = "stopSequences", skip_serializing_if = "Vec::is_empty")]
    stop_sequences: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
    #[serde(rename = "modelVersion")]
    model_version: Option<String>,
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    code: Option<i32>,
    message: Option<String>,
    status: Option<String>,
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    async fn chat(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Build contents array (conversation history)
        let mut contents: Vec<GeminiContent> = Vec::new();

        // Convert messages to Gemini format
        for m in request.messages {
            let role = match m.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "model",
                MessageRole::System => "user", // System messages go to systemInstruction
            };
            
            // Skip system messages in contents (handled separately)
            if matches!(m.role, MessageRole::System) {
                continue;
            }

            contents.push(GeminiContent {
                role: Some(role.to_string()),
                parts: vec![GeminiPart { text: m.content }],
            });
        }

        // Build system instruction if provided
        let system_instruction = request.system.map(|text| GeminiContent {
            role: None,
            parts: vec![GeminiPart { text }],
        });

        let gemini_request = GeminiRequest {
            contents,
            system_instruction,
            generation_config: GeminiGenerationConfig {
                max_output_tokens: request.max_tokens.min(self.max_tokens),
                temperature: request.temperature.min(self.temperature.max(0.0)),
                stop_sequences: request.stop_sequences,
            },
        };

        // Gemini API URL
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| OfficeError::LlmError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OfficeError::LlmError(format!("API error: {}", error_text)));
        }

        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| OfficeError::LlmError(format!("Parse failed: {}", e)))?;

        // Check for error in response
        if let Some(error) = gemini_response.error {
            return Err(OfficeError::LlmError(format!(
                "Gemini error: {} ({})",
                error.message.unwrap_or_default(),
                error.status.unwrap_or_default()
            )));
        }

        let candidates = gemini_response.candidates
            .ok_or_else(|| OfficeError::LlmError("No candidates returned".to_string()))?;

        let candidate = candidates.into_iter().next()
            .ok_or_else(|| OfficeError::LlmError("Empty candidates array".to_string()))?;

        let content = candidate.content.parts.into_iter()
            .map(|p| p.text)
            .collect::<Vec<_>>()
            .join("");

        let usage = gemini_response.usage_metadata.unwrap_or(GeminiUsageMetadata {
            prompt_token_count: Some(0),
            candidates_token_count: Some(0),
            total_token_count: Some(0),
        });

        Ok(LlmResponse {
            content,
            finish_reason: candidate.finish_reason.unwrap_or_else(|| "STOP".to_string()),
            usage: LlmUsage {
                input_tokens: usage.prompt_token_count.unwrap_or(0),
                output_tokens: usage.candidates_token_count.unwrap_or(0),
                total_tokens: usage.total_token_count.unwrap_or(0),
            },
            model: gemini_response.model_version.unwrap_or_else(|| self.model.clone()),
        })
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}
