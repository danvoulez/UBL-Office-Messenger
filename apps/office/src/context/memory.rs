//! Memory - Hybrid Memory Strategy
//!
//! Implements the hybrid memory strategy:
//! - Recent events: verbatim (last 20)
//! - Historical periods: synthesized
//! - Bookmarks: important events
//! - Baseline: consolidated narrative

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Number of recent events to keep verbatim
    pub recent_event_count: usize,
    /// Number of days for recent synthesis
    pub recent_synthesis_days: u32,
    /// Maximum bookmarks
    pub max_bookmarks: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            recent_event_count: 20,
            recent_synthesis_days: 7,
            max_bookmarks: 50,
        }
    }
}

/// Memory strategy for context building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStrategy {
    /// Full verbatim (for small histories)
    Full,
    /// Hybrid (recent + synthesis + bookmarks + baseline)
    Hybrid,
    /// Compressed (baseline + bookmarks only)
    Compressed,
    /// Minimal (baseline only)
    Minimal,
}

/// A memory entry (event from the ledger)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Event ID (from ledger)
    pub event_id: String,
    /// Event type
    pub event_type: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event summary
    pub summary: String,
    /// Full event data (for verbatim entries)
    pub data: Option<serde_json::Value>,
    /// Whether this is bookmarked
    pub is_bookmarked: bool,
}

/// A bookmark - important event to remember
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Event ID
    pub event_id: String,
    /// Bookmark reason
    pub reason: String,
    /// Bookmark timestamp
    pub created_at: DateTime<Utc>,
    /// Event summary
    pub event_summary: String,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Historical synthesis (compressed history)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSynthesis {
    /// Period start
    pub period_start: DateTime<Utc>,
    /// Period end
    pub period_end: DateTime<Utc>,
    /// Synthesized narrative
    pub narrative: String,
    /// Key events count
    pub event_count: u32,
    /// Key themes
    pub themes: Vec<String>,
}

/// Memory state for an entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Memory {
    /// Recent events (verbatim, most recent first)
    pub recent_events: Vec<MemoryEntry>,
    /// Historical syntheses
    pub historical_syntheses: Vec<HistoricalSynthesis>,
    /// Bookmarks
    pub bookmarks: Vec<Bookmark>,
    /// Baseline narrative (from dreaming)
    pub baseline_narrative: String,
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
    /// Total events seen
    pub total_events: u64,
}

impl Memory {
    /// Create new memory with baseline
    pub fn new(baseline_narrative: String) -> Self {
        Self {
            baseline_narrative,
            last_updated: Some(Utc::now()),
            ..Default::default()
        }
    }

    /// Add a recent event
    pub fn add_event(&mut self, event: MemoryEntry, config: &MemoryConfig) {
        self.recent_events.insert(0, event);
        self.total_events += 1;

        // Trim to configured limit
        if self.recent_events.len() > config.recent_event_count {
            self.recent_events.truncate(config.recent_event_count);
        }

        self.last_updated = Some(Utc::now());
    }

    /// Add a bookmark
    pub fn add_bookmark(&mut self, bookmark: Bookmark, config: &MemoryConfig) {
        // Check if already bookmarked
        if self.bookmarks.iter().any(|b| b.event_id == bookmark.event_id) {
            return;
        }

        self.bookmarks.push(bookmark);

        // Sort by creation time (most recent first)
        self.bookmarks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Trim to limit
        if self.bookmarks.len() > config.max_bookmarks {
            self.bookmarks.truncate(config.max_bookmarks);
        }
    }

    /// Remove a bookmark
    pub fn remove_bookmark(&mut self, event_id: &str) {
        self.bookmarks.retain(|b| b.event_id != event_id);
    }

    /// Add a historical synthesis
    pub fn add_synthesis(&mut self, synthesis: HistoricalSynthesis) {
        self.historical_syntheses.push(synthesis);

        // Sort by period end (most recent first)
        self.historical_syntheses.sort_by(|a, b| b.period_end.cmp(&a.period_end));
    }

    /// Update baseline narrative
    pub fn update_baseline(&mut self, narrative: String) {
        self.baseline_narrative = narrative;
        self.last_updated = Some(Utc::now());
    }

    /// Get token estimate for memory content
    pub fn estimate_tokens(&self) -> u64 {
        // Rough estimate: 4 characters per token
        let mut chars = 0usize;

        chars += self.baseline_narrative.len();

        for event in &self.recent_events {
            chars += event.summary.len();
            if let Some(data) = &event.data {
                if let Ok(json) = serde_json::to_string(data) {
                    chars += json.len();
                }
            }
        }

        for synthesis in &self.historical_syntheses {
            chars += synthesis.narrative.len();
        }

        for bookmark in &self.bookmarks {
            chars += bookmark.event_summary.len();
            chars += bookmark.reason.len();
        }

        (chars / 4) as u64
    }

    /// Compress memory to fit within token budget
    pub fn compress_to_budget(&mut self, max_tokens: u64) {
        while self.estimate_tokens() > max_tokens {
            // First, remove data from recent events
            let mut cleared_any = false;
            for event in &mut self.recent_events {
                if event.data.is_some() {
                    event.data = None;
                    cleared_any = true;
                }
            }
            if cleared_any && self.estimate_tokens() <= max_tokens {
                return;
            }

            // Then, reduce recent events
            if self.recent_events.len() > 5 {
                self.recent_events.pop();
                continue;
            }

            // Then, reduce historical syntheses
            if self.historical_syntheses.len() > 1 {
                self.historical_syntheses.pop();
                continue;
            }

            // Finally, truncate baseline
            if self.baseline_narrative.len() > 500 {
                self.baseline_narrative = self.baseline_narrative[..500].to_string() + "...";
            }

            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new("Initial baseline narrative.".to_string());
        assert!(memory.recent_events.is_empty());
        assert!(memory.bookmarks.is_empty());
        assert_eq!(memory.total_events, 0);
    }

    #[test]
    fn test_add_events() {
        let mut memory = Memory::new("Baseline".to_string());
        let config = MemoryConfig {
            recent_event_count: 5,
            ..Default::default()
        };

        for i in 0..10 {
            let event = MemoryEntry {
                event_id: format!("event_{}", i),
                event_type: "test".to_string(),
                timestamp: Utc::now(),
                summary: format!("Event {}", i),
                data: None,
                is_bookmarked: false,
            };
            memory.add_event(event, &config);
        }

        assert_eq!(memory.recent_events.len(), 5);
        assert_eq!(memory.total_events, 10);
        // Most recent should be first
        assert_eq!(memory.recent_events[0].event_id, "event_9");
    }

    #[test]
    fn test_bookmarks() {
        let mut memory = Memory::new("Baseline".to_string());
        let config = MemoryConfig::default();

        let bookmark = Bookmark {
            event_id: "important_event".to_string(),
            reason: "Critical decision".to_string(),
            created_at: Utc::now(),
            event_summary: "Made an important decision".to_string(),
            tags: vec!["decision".to_string()],
        };

        memory.add_bookmark(bookmark.clone(), &config);
        assert_eq!(memory.bookmarks.len(), 1);

        // Adding same bookmark should not duplicate
        memory.add_bookmark(bookmark, &config);
        assert_eq!(memory.bookmarks.len(), 1);

        memory.remove_bookmark("important_event");
        assert!(memory.bookmarks.is_empty());
    }

    #[test]
    fn test_token_estimation() {
        let mut memory = Memory::new("A baseline narrative with some content.".to_string());
        let initial_tokens = memory.estimate_tokens();

        let config = MemoryConfig::default();
        let event = MemoryEntry {
            event_id: "test".to_string(),
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            summary: "This is a test event with some content.".to_string(),
            data: None,
            is_bookmarked: false,
        };

        memory.add_event(event, &config);
        assert!(memory.estimate_tokens() > initial_tokens);
    }
}
