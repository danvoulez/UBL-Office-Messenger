//! Local/Mock LLM Provider

use async_trait::async_trait;

use super::provider::{LlmProvider, LlmRequest, LlmResponse, LlmUsage};
use crate::Result;

/// Local provider for testing
pub struct LocalProvider {
    response_prefix: String,
}

impl LocalProvider {
    pub fn new() -> Self {
        Self {
            response_prefix: "[LOCAL LLM] ".to_string(),
        }
    }

    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            response_prefix: prefix.into(),
        }
    }
}

impl Default for LocalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for LocalProvider {
    fn name(&self) -> &str {
        "local"
    }

    async fn chat(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Generate a mock response based on the last user message
        let last_user_message = request.messages
            .iter()
            .rev()
            .find(|m| m.role == super::provider::MessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "No message provided".to_string());

        let content = format!(
            "{}Received your message: '{}'. \
            This is a mock response from the local LLM provider. \
            In production, this would be handled by a real LLM.",
            self.response_prefix,
            if last_user_message.len() > 100 {
                format!("{}...", &last_user_message[..100])
            } else {
                last_user_message.clone()
            }
        );

        let input_tokens = request.messages
            .iter()
            .map(|m| m.content.len() / 4)
            .sum::<usize>() as u32;

        let output_tokens = (content.len() / 4) as u32;

        Ok(LlmResponse {
            content,
            finish_reason: "stop".to_string(),
            usage: LlmUsage {
                input_tokens,
                output_tokens,
                total_tokens: input_tokens + output_tokens,
            },
            model: "local-mock-v1".to_string(),
        })
    }

    async fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::LlmMessage;

    #[tokio::test]
    async fn test_local_provider() {
        let provider = LocalProvider::new();

        let request = LlmRequest::new(vec![
            LlmMessage::user("Hello, world!"),
        ]);

        let response = provider.chat(request).await.unwrap();

        assert!(response.content.contains("Hello, world!"));
        assert_eq!(response.finish_reason, "stop");
        assert!(response.usage.total_tokens > 0);
    }
}
