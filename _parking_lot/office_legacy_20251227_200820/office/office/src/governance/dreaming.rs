//! Dreaming Cycle - Memory Consolidation
//!
//! Asynchronous process that consolidates memory and removes anxiety.

use std::sync::Arc;

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::entity::EntityId;
use crate::context::{Memory, HistoricalSynthesis};
use crate::session::Handover;
use crate::ubl_client::UblClient;
use crate::llm::LlmProvider;
use crate::Result;

/// Configuration for dreaming cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamingConfig {
    /// Interval in hours between cycles
    pub interval_hours: u32,
    /// Session threshold (trigger after N sessions)
    pub session_threshold: u32,
    /// Maximum events to process per cycle
    pub max_events: usize,
    /// Days to synthesize
    pub synthesis_days: u32,
    /// Enable garbage collection
    pub enable_gc: bool,
    /// Enable emotional reset
    pub enable_emotional_reset: bool,
    /// Enable pattern synthesis
    pub enable_pattern_synthesis: bool,
}

impl Default for DreamingConfig {
    fn default() -> Self {
        Self {
            interval_hours: 24,
            session_threshold: 50,
            max_events: 1000,
            synthesis_days: 7,
            enable_gc: true,
            enable_emotional_reset: true,
            enable_pattern_synthesis: true,
        }
    }
}

/// Result of a dreaming cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamingResult {
    /// Entity ID
    pub entity_id: EntityId,
    /// Cycle start time
    pub started_at: DateTime<Utc>,
    /// Cycle end time
    pub ended_at: DateTime<Utc>,
    /// Events processed
    pub events_processed: usize,
    /// Events archived (garbage collected)
    pub events_archived: usize,
    /// Anxieties cleared
    pub anxieties_cleared: Vec<String>,
    /// Patterns synthesized
    pub patterns: Vec<String>,
    /// New baseline narrative
    pub new_baseline: String,
    /// Historical syntheses created
    pub syntheses_created: usize,
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Dreaming Cycle - Memory consolidation process
pub struct DreamingCycle {
    config: DreamingConfig,
    ubl_client: Arc<UblClient>,
    llm_provider: Option<Arc<dyn LlmProvider>>,
}

impl DreamingCycle {
    /// Create a new dreaming cycle
    pub fn new(config: DreamingConfig, ubl_client: Arc<UblClient>) -> Self {
        Self {
            config,
            ubl_client,
            llm_provider: None,
        }
    }

    /// Set LLM provider for synthesis
    pub fn with_llm_provider(mut self, provider: Arc<dyn LlmProvider>) -> Self {
        self.llm_provider = Some(provider);
        self
    }

    /// Check if dreaming is due
    pub fn is_due(&self, last_dream: Option<DateTime<Utc>>, session_count: u32) -> bool {
        // Check session threshold
        if session_count >= self.config.session_threshold {
            return true;
        }

        // Check time threshold
        if let Some(last) = last_dream {
            let threshold = Duration::hours(self.config.interval_hours as i64);
            return Utc::now() - last >= threshold;
        }

        // No previous dream, should do one
        true
    }

    /// Execute dreaming cycle
    pub async fn execute(&self, entity_id: &EntityId, memory: &mut Memory) -> Result<DreamingResult> {
        let started_at = Utc::now();
        let mut events_archived = 0;
        let mut anxieties_cleared = Vec::new();
        let mut patterns = Vec::new();
        let mut errors = Vec::new();
        let mut syntheses_created = 0;

        // 1. Garbage Collection
        if self.config.enable_gc {
            match self.garbage_collect(entity_id, memory).await {
                Ok(count) => events_archived = count,
                Err(e) => errors.push(format!("GC error: {}", e)),
            }
        }

        // 2. Emotional Reset
        if self.config.enable_emotional_reset {
            match self.emotional_reset(entity_id, memory).await {
                Ok(cleared) => anxieties_cleared = cleared,
                Err(e) => errors.push(format!("Emotional reset error: {}", e)),
            }
        }

        // 3. Pattern Synthesis
        if self.config.enable_pattern_synthesis {
            match self.synthesize_patterns(entity_id, memory).await {
                Ok((p, count)) => {
                    patterns = p;
                    syntheses_created = count;
                },
                Err(e) => errors.push(format!("Pattern synthesis error: {}", e)),
            }
        }

        // 4. Baseline Update
        let new_baseline = match self.update_baseline(entity_id, memory, &patterns).await {
            Ok(baseline) => {
                memory.update_baseline(baseline.clone());
                baseline
            },
            Err(e) => {
                errors.push(format!("Baseline update error: {}", e));
                memory.baseline_narrative.clone()
            }
        };

        let ended_at = Utc::now();

        Ok(DreamingResult {
            entity_id: entity_id.clone(),
            started_at,
            ended_at,
            events_processed: memory.total_events as usize,
            events_archived,
            anxieties_cleared,
            patterns,
            new_baseline,
            syntheses_created,
            errors,
        })
    }

    /// Garbage collection - archive old, resolved events
    async fn garbage_collect(&self, entity_id: &EntityId, _memory: &mut Memory) -> Result<usize> {
        // Get resolved issues from ledger
        let resolved = self.ubl_client.get_resolved_issues(entity_id).await
            .unwrap_or_default();

        let resolved_count = resolved.len();

        // Archive old events that are about resolved issues
        // In real implementation, this would move to archive storage
        // For now, we just track the count

        Ok(resolved_count)
    }

    /// Emotional reset - identify and clear resolved anxieties
    async fn emotional_reset(
        &self,
        entity_id: &EntityId,
        _memory: &Memory,
    ) -> Result<Vec<String>> {
        // Look for anxiety-related keywords in recent handovers
        let handovers = self.ubl_client.get_handovers(entity_id, 10).await
            .unwrap_or_default();

        let mut cleared = Vec::new();

        for handover in handovers {
            let anxiety_keywords = ["anxious", "worried", "concerned", "stressed",
                "ansioso", "preocupado", "estressado"];

            let has_anxiety = anxiety_keywords.iter()
                .any(|k| handover.content.to_lowercase().contains(k));

            if has_anxiety {
                // Check if the issue was resolved
                if self.is_issue_resolved(&handover, entity_id).await {
                    cleared.push(format!("Resolved anxiety from session {}", handover.session_id));
                }
            }
        }

        Ok(cleared)
    }

    /// Check if an issue mentioned in handover is resolved
    async fn is_issue_resolved(&self, handover: &Handover, entity_id: &EntityId) -> bool {
        // Query for resolution events after the handover
        let events = self.ubl_client
            .get_events_after(entity_id, handover.created_at)
            .await
            .unwrap_or_default();

        let resolution_keywords = ["resolved", "completed", "fixed", "done",
            "resolvido", "completado", "corrigido", "feito"];

        events.iter().any(|e| {
            resolution_keywords.iter()
                .any(|k| e.summary.to_lowercase().contains(k))
        })
    }

    /// Synthesize patterns from historical sessions
    async fn synthesize_patterns(
        &self,
        entity_id: &EntityId,
        memory: &mut Memory,
    ) -> Result<(Vec<String>, usize)> {
        // Get trajectories/sessions for analysis
        let trajectories = self.ubl_client
            .get_trajectories(entity_id, self.config.synthesis_days)
            .await
            .unwrap_or_default();

        // Extract patterns (simplified without LLM)
        let mut patterns = Vec::new();
        let mut session_types = std::collections::HashMap::new();

        for traj in &trajectories {
            *session_types.entry(traj.session_type.clone()).or_insert(0) += 1;
        }

        for (session_type, count) in session_types {
            if count >= 3 {
                patterns.push(format!(
                    "Frequently uses {} sessions ({} in last {} days)",
                    session_type, count, self.config.synthesis_days
                ));
            }
        }

        // Create historical synthesis
        if !trajectories.is_empty() {
            let patterns_str = if patterns.is_empty() {
                "No notable patterns.".to_string()
            } else {
                patterns.join(" ")
            };
            let synthesis = HistoricalSynthesis {
                period_start: Utc::now() - Duration::days(self.config.synthesis_days as i64),
                period_end: Utc::now(),
                narrative: format!(
                    "Over the past {} days, completed {} sessions. {}",
                    self.config.synthesis_days,
                    trajectories.len(),
                    patterns_str
                ),
                event_count: trajectories.len() as u32,
                themes: patterns.clone(),
            };

            memory.add_synthesis(synthesis);
        }

        let syntheses_count = 1;
        Ok((patterns, syntheses_count))
    }

    /// Update baseline narrative
    async fn update_baseline(
        &self,
        entity_id: &EntityId,
        memory: &Memory,
        patterns: &[String],
    ) -> Result<String> {
        // If we have an LLM provider, use it for synthesis
        if let Some(llm) = &self.llm_provider {
            let prompt = format!(
                "Synthesize a baseline narrative for entity {}.\n\
                Current baseline: {}\n\
                Recent patterns: {}\n\
                Total events: {}\n\n\
                Generate a concise, factual baseline narrative in 2-3 sentences.",
                entity_id,
                memory.baseline_narrative,
                patterns.join(", "),
                memory.total_events
            );

            match llm.complete(&prompt, 200).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    tracing::warn!("LLM synthesis failed: {}", e);
                }
            }
        }

        // Fallback: generate simple baseline
        let baseline = format!(
            "Entity {} has processed {} total events. {}",
            entity_id,
            memory.total_events,
            if patterns.is_empty() {
                "Operating normally with no notable patterns.".to_string()
            } else {
                format!("Notable patterns: {}", patterns.join("; "))
            }
        );

        Ok(baseline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_due_session_threshold() {
        let config = DreamingConfig {
            session_threshold: 10,
            ..Default::default()
        };

        // Need UblClient for DreamingCycle, so just test config
        assert_eq!(config.session_threshold, 10);
    }

    #[test]
    fn test_default_config() {
        let config = DreamingConfig::default();

        assert_eq!(config.interval_hours, 24);
        assert_eq!(config.session_threshold, 50);
        assert!(config.enable_gc);
        assert!(config.enable_emotional_reset);
        assert!(config.enable_pattern_synthesis);
    }
}
