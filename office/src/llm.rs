//! LLM Client with JSON Retry Loop (Gemini P1 #5)
//!
//! Problem: LLMs sometimes produce invalid JSON (missing commas, wrong quotes, etc.)
//! Solution: Parse the JSON, and if it fails, send the error back to the LLM
//! to self-correct (up to MAX_RETRIES times).

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, warn, error};

/// Maximum JSON parse retries
pub const MAX_JSON_RETRIES: u32 = 3;

/// LLM provider trait
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Complete a prompt with the LLM
    async fn complete(&self, messages: &[Message], options: &CompletionOptions) -> Result<String, LlmError>;
    
    /// Get the provider name
    fn name(&self) -> &'static str;
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Completion options
#[derive(Debug, Clone, Default)]
pub struct CompletionOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub json_mode: bool,
}

/// LLM error types
#[derive(Debug, Clone)]
pub enum LlmError {
    NetworkError(String),
    RateLimited { retry_after_ms: u64 },
    InvalidResponse(String),
    JsonParseError { raw: String, error: String },
    MaxRetriesExceeded { last_error: String },
    Timeout,
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::NetworkError(e) => write!(f, "Network error: {}", e),
            LlmError::RateLimited { retry_after_ms } => {
                write!(f, "Rate limited, retry after {}ms", retry_after_ms)
            }
            LlmError::InvalidResponse(e) => write!(f, "Invalid response: {}", e),
            LlmError::JsonParseError { raw, error } => {
                write!(f, "JSON parse error: {} (raw: {}...)", error, &raw[..100.min(raw.len())])
            }
            LlmError::MaxRetriesExceeded { last_error } => {
                write!(f, "Max retries exceeded: {}", last_error)
            }
            LlmError::Timeout => write!(f, "Request timeout"),
        }
    }
}

impl std::error::Error for LlmError {}

/// LLM client with JSON retry logic
pub struct LlmClient<P: LlmProvider> {
    provider: P,
    default_options: CompletionOptions,
}

impl<P: LlmProvider> LlmClient<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            default_options: CompletionOptions::default(),
        }
    }

    pub fn with_options(mut self, options: CompletionOptions) -> Self {
        self.default_options = options;
        self
    }

    /// Complete a prompt and expect raw text
    pub async fn complete_text(
        &self,
        messages: &[Message],
        options: Option<&CompletionOptions>,
    ) -> Result<String, LlmError> {
        let opts = options.unwrap_or(&self.default_options);
        self.provider.complete(messages, opts).await
    }

    /// Complete a prompt and expect valid JSON
    /// Automatically retries with error feedback if JSON is invalid
    pub async fn complete_json<T: DeserializeOwned>(
        &self,
        mut messages: Vec<Message>,
        options: Option<&CompletionOptions>,
    ) -> Result<T, LlmError> {
        let opts = options.unwrap_or(&self.default_options);
        let mut last_error = String::new();

        for attempt in 0..=MAX_JSON_RETRIES {
            debug!("[LLM] JSON completion attempt {}/{}", attempt + 1, MAX_JSON_RETRIES + 1);
            
            let raw = self.provider.complete(&messages, opts).await?;
            
            // Extract JSON from response (handle markdown code blocks)
            let json_str = extract_json(&raw);
            
            match serde_json::from_str::<T>(&json_str) {
                Ok(parsed) => {
                    if attempt > 0 {
                        debug!("[LLM] JSON parsed successfully after {} retries", attempt);
                    }
                    return Ok(parsed);
                }
                Err(e) => {
                    last_error = format!("{}", e);
                    warn!(
                        "[LLM] JSON parse error (attempt {}): {}",
                        attempt + 1,
                        last_error
                    );

                    if attempt < MAX_JSON_RETRIES {
                        // Add error feedback for self-correction
                        messages.push(Message {
                            role: MessageRole::Assistant,
                            content: raw.clone(),
                        });
                        messages.push(Message {
                            role: MessageRole::User,
                            content: format!(
                                "Your response contained invalid JSON. Error: {}\n\n\
                                Please fix the JSON and respond with ONLY the corrected JSON, no explanation.",
                                last_error
                            ),
                        });
                    }
                }
            }
        }

        error!("[LLM] Max retries exceeded for JSON completion");
        Err(LlmError::MaxRetriesExceeded { last_error })
    }

    /// Complete a prompt expecting a specific JSON schema
    /// Includes schema hint in the prompt
    pub async fn complete_json_with_schema<T: DeserializeOwned>(
        &self,
        messages: Vec<Message>,
        schema_hint: &str,
        options: Option<&CompletionOptions>,
    ) -> Result<T, LlmError> {
        let mut messages_with_schema = messages;
        
        // Add schema hint to the last user message or as a new system message
        let schema_msg = Message {
            role: MessageRole::System,
            content: format!(
                "You must respond with valid JSON matching this schema:\n```json\n{}\n```\n\
                Respond ONLY with the JSON, no additional text or markdown.",
                schema_hint
            ),
        };
        
        // Insert schema hint before the last message
        let len = messages_with_schema.len();
        if len > 0 {
            messages_with_schema.insert(len - 1, schema_msg);
        } else {
            messages_with_schema.push(schema_msg);
        }

        self.complete_json(messages_with_schema, options).await
    }
}

/// Extract JSON from a string that might have markdown code blocks
fn extract_json(s: &str) -> String {
    let trimmed = s.trim();
    
    // Check for ```json ... ``` blocks
    if let Some(start) = trimmed.find("```json") {
        let after_fence = &trimmed[start + 7..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim().to_string();
        }
    }
    
    // Check for ``` ... ``` blocks
    if let Some(start) = trimmed.find("```") {
        let after_fence = &trimmed[start + 3..];
        if let Some(end) = after_fence.find("```") {
            let content = after_fence[..end].trim();
            // Skip language identifier if present
            if let Some(first_newline) = content.find('\n') {
                let first_line = &content[..first_newline];
                if !first_line.starts_with('{') && !first_line.starts_with('[') {
                    return content[first_newline + 1..].trim().to_string();
                }
            }
            return content.to_string();
        }
    }
    
    // Look for JSON object or array
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return trimmed[start..=end].to_string();
        }
    }
    
    if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            return trimmed[start..=end].to_string();
        }
    }
    
    // Return as-is if no JSON found
    trimmed.to_string()
}

// =============================================================================
// MOCK PROVIDER (for testing)
// =============================================================================

pub struct MockLlmProvider {
    responses: std::sync::Mutex<Vec<String>>,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self {
            responses: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn add_response(&self, response: String) {
        self.responses.lock().unwrap().push(response);
    }
}

#[async_trait::async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(&self, _messages: &[Message], _options: &CompletionOptions) -> Result<String, LlmError> {
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Ok(r#"{"status": "ok"}"#.to_string())
        } else {
            Ok(responses.remove(0))
        }
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_plain() {
        let input = r#"{"key": "value"}"#;
        assert_eq!(extract_json(input), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_extract_json_markdown() {
        let input = r#"Here's the JSON:
```json
{"key": "value"}
```
That's it."#;
        assert_eq!(extract_json(input), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_extract_json_code_block() {
        let input = r#"```
{"key": "value"}
```"#;
        assert_eq!(extract_json(input), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_extract_json_with_text() {
        let input = r#"The result is {"key": "value"} as expected."#;
        assert_eq!(extract_json(input), r#"{"key": "value"}"#);
    }

    #[tokio::test]
    async fn test_json_retry_success() {
        let provider = MockLlmProvider::new();
        // First response is invalid, second is valid
        provider.add_response(r#"{"key": "value""#.to_string()); // Missing }
        provider.add_response(r#"{"key": "value"}"#.to_string());

        let client = LlmClient::new(provider);
        let result: serde_json::Value = client
            .complete_json(vec![Message {
                role: MessageRole::User,
                content: "test".to_string(),
            }], None)
            .await
            .unwrap();

        assert_eq!(result["key"], "value");
    }
}

