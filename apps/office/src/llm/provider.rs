//! LLM Provider Trait

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

/// Role of a message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: MessageRole,
    pub content: String,
}

impl LlmMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Request to LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Messages in conversation
    pub messages: Vec<LlmMessage>,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature (0.0 to 1.0)
    pub temperature: f32,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    /// System prompt (for providers that support it separately)
    pub system: Option<String>,
}

impl LlmRequest {
    pub fn new(messages: Vec<LlmMessage>) -> Self {
        Self {
            messages,
            max_tokens: 4096,
            temperature: 0.7,
            stop_sequences: vec![],
            system: None,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn with_stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop_sequences = sequences;
        self
    }
}

/// Response from LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Generated content
    pub content: String,
    /// Finish reason
    pub finish_reason: String,
    /// Usage statistics
    pub usage: LlmUsage,
    /// Model used
    pub model: String,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmUsage {
    /// Input tokens
    pub input_tokens: u32,
    /// Output tokens
    pub output_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Send a request and get a response
    async fn chat(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Simple completion (convenience method)
    async fn complete(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        let request = LlmRequest::new(vec![LlmMessage::user(prompt)])
            .with_max_tokens(max_tokens);
        let response = self.chat(request).await?;
        Ok(response.content)
    }

    /// Check if provider is available
    async fn is_available(&self) -> bool {
        true
    }
}
