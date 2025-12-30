//! LLM Provider Module
//!
//! Abstractions for LLM providers (Anthropic, OpenAI, Gemini, etc.)
//!
//! Architecture:
//! - Providers are "dumb pipes" that call LLM APIs
//! - SmartRouter selects the best provider for each task
//! - All routing logic lives in OFFICE (the brain)

mod provider;
mod anthropic;
mod openai;
mod gemini;
mod local;
mod router;

pub use provider::{LlmProvider, LlmRequest, LlmResponse, LlmMessage, LlmUsage, MessageRole};
pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;
pub use gemini::GeminiProvider;
pub use local::LocalProvider;
pub use router::{SmartRouter, TaskType, RoutingPreferences, ProviderProfile, default_profiles};

use std::sync::Arc;

use crate::{LlmConfig, OfficeError, Result};

/// Create an LLM provider from configuration
pub fn create_provider(config: &LlmConfig) -> Result<Arc<dyn LlmProvider>> {
    match config.provider.to_lowercase().as_str() {
        "anthropic" | "claude" => {
            Ok(Arc::new(AnthropicProvider::new(
                &config.api_key,
                &config.model,
                config.max_tokens,
                config.temperature,
            )))
        }
        "openai" | "gpt" => {
            Ok(Arc::new(OpenAIProvider::new(
                &config.api_key,
                &config.model,
                config.max_tokens,
                config.temperature,
            )))
        }
        "gemini" | "google" => {
            Ok(Arc::new(GeminiProvider::new(
                &config.api_key,
                &config.model,
                config.max_tokens,
                config.temperature,
            )))
        }
        "local" | "mock" => {
            Ok(Arc::new(LocalProvider::new()))
        }
        _ => Err(OfficeError::ConfigError(format!(
            "Unknown LLM provider: {}",
            config.provider
        ))),
    }
}
