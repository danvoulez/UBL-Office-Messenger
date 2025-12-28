//! Smart Router - Intelligent LLM Provider Selection
//!
//! Routes tasks to the optimal LLM provider based on:
//! - Task type (coding, writing, analysis, creative)
//! - Entity preferences
//! - Cost/speed tradeoffs
//! - Provider availability
//!
//! The Router lives in OFFICE because OFFICE knows the context.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::provider::{LlmProvider, LlmRequest, LlmResponse};
use crate::{OfficeError, Result};

/// Task types for routing decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// Code generation, debugging, refactoring
    Coding,
    /// Text writing, documentation
    Writing,
    /// Data analysis, reasoning
    Analysis,
    /// Creative tasks, brainstorming
    Creative,
    /// Simple quick responses
    Quick,
    /// Complex multi-step reasoning
    Complex,
    /// Unknown - use default routing
    Unknown,
}

impl Default for TaskType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Routing preferences
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingPreferences {
    /// Prefer speed over quality
    pub prefer_speed: bool,
    /// Prefer cost savings
    pub prefer_economy: bool,
    /// Preferred provider (if specified)
    pub preferred_provider: Option<String>,
    /// Maximum cost (in cents) for this request
    pub max_cost_cents: Option<u32>,
    /// Maximum latency (in ms) acceptable
    pub max_latency_ms: Option<u32>,
}

/// Provider capabilities and scoring
#[derive(Debug, Clone)]
pub struct ProviderProfile {
    /// Provider name
    pub name: String,
    /// Task type scores (0-100, higher is better)
    pub task_scores: HashMap<TaskType, u8>,
    /// Average latency in ms
    pub avg_latency_ms: u32,
    /// Cost per 1M tokens (input + output average)
    pub cost_per_million_tokens: f32,
    /// Is currently available
    pub available: bool,
}

impl ProviderProfile {
    /// Get score for a task type
    pub fn score_for_task(&self, task: TaskType) -> u8 {
        *self.task_scores.get(&task).unwrap_or(&50)
    }

    /// Calculate total score with preferences
    pub fn calculate_score(&self, task: TaskType, prefs: &RoutingPreferences) -> f32 {
        let base_score = self.score_for_task(task) as f32;
        
        // Speed bonus (inverse of latency)
        let speed_modifier = if prefs.prefer_speed {
            (1000.0 / self.avg_latency_ms as f32).min(2.0)
        } else {
            1.0
        };
        
        // Economy bonus (inverse of cost)
        let economy_modifier = if prefs.prefer_economy {
            (10.0 / self.cost_per_million_tokens).min(2.0)
        } else {
            1.0
        };
        
        base_score * speed_modifier * economy_modifier
    }
}

/// Smart Router for LLM provider selection
pub struct SmartRouter {
    /// Available providers
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    /// Provider profiles for routing
    profiles: HashMap<String, ProviderProfile>,
    /// Default provider
    default_provider: String,
}

impl SmartRouter {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            profiles: HashMap::new(),
            default_provider: String::new(),
        }
    }

    /// Register a provider with its profile
    pub fn register(&mut self, provider: Arc<dyn LlmProvider>, profile: ProviderProfile) {
        let name = profile.name.clone();
        self.providers.insert(name.clone(), provider);
        
        // Set as default if first provider
        if self.default_provider.is_empty() {
            self.default_provider = name.clone();
        }
        
        self.profiles.insert(name, profile);
    }

    /// Set the default provider
    pub fn set_default(&mut self, name: &str) {
        if self.providers.contains_key(name) {
            self.default_provider = name.to_string();
        }
    }

    /// Route a request to the best provider
    pub async fn route(
        &self,
        request: LlmRequest,
        task: TaskType,
        prefs: &RoutingPreferences,
    ) -> Result<LlmResponse> {
        let provider = self.select_provider(task, prefs).await?;
        provider.chat(request).await
    }

    /// Select the best provider for a task
    pub async fn select_provider(
        &self,
        task: TaskType,
        prefs: &RoutingPreferences,
    ) -> Result<Arc<dyn LlmProvider>> {
        // Check for explicit preference
        if let Some(ref preferred) = prefs.preferred_provider {
            if let Some(provider) = self.providers.get(preferred) {
                if provider.is_available().await {
                    return Ok(provider.clone());
                }
            }
        }

        // Score all available providers
        let mut best_provider: Option<(&str, f32)> = None;

        for (name, profile) in &self.profiles {
            // Skip unavailable providers
            if !profile.available {
                continue;
            }

            // Check latency constraint
            if let Some(max_latency) = prefs.max_latency_ms {
                if profile.avg_latency_ms > max_latency {
                    continue;
                }
            }

            let score = profile.calculate_score(task, prefs);
            
            if best_provider.is_none() || score > best_provider.unwrap().1 {
                best_provider = Some((name.as_str(), score));
            }
        }

        // Get the best provider or fall back to default
        let provider_name = best_provider
            .map(|(name, _)| name)
            .unwrap_or(&self.default_provider);

        self.providers
            .get(provider_name)
            .cloned()
            .ok_or_else(|| OfficeError::LlmError("No provider available".to_string()))
    }

    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.get(name).cloned()
    }

    /// Check if any provider is available
    pub async fn is_available(&self) -> bool {
        for provider in self.providers.values() {
            if provider.is_available().await {
                return true;
            }
        }
        false
    }
}

impl Default for SmartRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default provider profiles
pub fn default_profiles() -> HashMap<String, ProviderProfile> {
    let mut profiles = HashMap::new();

    // Claude (Anthropic) - Great for coding, analysis, complex reasoning
    let mut claude_scores = HashMap::new();
    claude_scores.insert(TaskType::Coding, 95);
    claude_scores.insert(TaskType::Analysis, 90);
    claude_scores.insert(TaskType::Complex, 95);
    claude_scores.insert(TaskType::Writing, 85);
    claude_scores.insert(TaskType::Creative, 80);
    claude_scores.insert(TaskType::Quick, 70);
    claude_scores.insert(TaskType::Unknown, 85);

    profiles.insert("anthropic".to_string(), ProviderProfile {
        name: "anthropic".to_string(),
        task_scores: claude_scores,
        avg_latency_ms: 2000,
        cost_per_million_tokens: 15.0, // ~$15/M tokens
        available: true,
    });

    // GPT-4 (OpenAI) - Strong all-around
    let mut gpt4_scores = HashMap::new();
    gpt4_scores.insert(TaskType::Coding, 90);
    gpt4_scores.insert(TaskType::Analysis, 85);
    gpt4_scores.insert(TaskType::Complex, 90);
    gpt4_scores.insert(TaskType::Writing, 90);
    gpt4_scores.insert(TaskType::Creative, 85);
    gpt4_scores.insert(TaskType::Quick, 75);
    gpt4_scores.insert(TaskType::Unknown, 85);

    profiles.insert("openai".to_string(), ProviderProfile {
        name: "openai".to_string(),
        task_scores: gpt4_scores,
        avg_latency_ms: 1500,
        cost_per_million_tokens: 30.0, // ~$30/M tokens
        available: true,
    });

    // Local/Mock - Fast but less capable
    let mut local_scores = HashMap::new();
    local_scores.insert(TaskType::Coding, 40);
    local_scores.insert(TaskType::Analysis, 40);
    local_scores.insert(TaskType::Complex, 30);
    local_scores.insert(TaskType::Writing, 50);
    local_scores.insert(TaskType::Creative, 45);
    local_scores.insert(TaskType::Quick, 80);
    local_scores.insert(TaskType::Unknown, 45);

    profiles.insert("local".to_string(), ProviderProfile {
        name: "local".to_string(),
        task_scores: local_scores,
        avg_latency_ms: 100,
        cost_per_million_tokens: 0.0, // Free
        available: true,
    });

    profiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_scoring() {
        let mut scores = HashMap::new();
        scores.insert(TaskType::Coding, 90);
        scores.insert(TaskType::Quick, 70);

        let profile = ProviderProfile {
            name: "test".to_string(),
            task_scores: scores,
            avg_latency_ms: 1000,
            cost_per_million_tokens: 10.0,
            available: true,
        };

        assert_eq!(profile.score_for_task(TaskType::Coding), 90);
        assert_eq!(profile.score_for_task(TaskType::Quick), 70);
        assert_eq!(profile.score_for_task(TaskType::Creative), 50); // Default
    }

    #[test]
    fn test_score_with_preferences() {
        let mut scores = HashMap::new();
        scores.insert(TaskType::Coding, 80);

        let profile = ProviderProfile {
            name: "test".to_string(),
            task_scores: scores,
            avg_latency_ms: 1000,
            cost_per_million_tokens: 10.0,
            available: true,
        };

        // No preferences
        let prefs = RoutingPreferences::default();
        let score = profile.calculate_score(TaskType::Coding, &prefs);
        assert_eq!(score, 80.0);

        // Prefer speed
        let speed_prefs = RoutingPreferences {
            prefer_speed: true,
            ..Default::default()
        };
        let score = profile.calculate_score(TaskType::Coding, &speed_prefs);
        assert!(score > 80.0); // Should be higher
    }
}

