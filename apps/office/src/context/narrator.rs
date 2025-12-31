//! Narrator - Data to Narrative Transformer
//!
//! Transforms structured context frames into situated first-person narratives.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::frame::{ContextFrame, Obligation, ObligationStatus};
use crate::session::SessionType;

/// Tool information for narrative injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Full name including server prefix (e.g., "office:ubl_query")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Parameter schema
    pub parameters: Option<Value>,
}

/// Configuration for narrative generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeConfig {
    /// Include full event data
    pub include_event_data: bool,
    /// Maximum events to include verbatim
    pub max_verbatim_events: usize,
    /// Include affordance details
    pub include_affordance_details: bool,
    /// Language for narrative
    pub language: String,
    /// Include MCP tool orientation
    pub include_tool_orientation: bool,
    /// Show detailed tool parameters
    pub show_tool_parameters: bool,
}

impl Default for NarrativeConfig {
    fn default() -> Self {
        Self {
            include_event_data: false,
            max_verbatim_events: 10,
            include_affordance_details: true,
            language: "en".to_string(),
            include_tool_orientation: true,
            show_tool_parameters: false,
        }
    }
}

/// Narrator - Generates situated narratives from context frames
pub struct Narrator {
    config: NarrativeConfig,
    /// Available MCP tools (injected when generating)
    tools: Vec<ToolInfo>,
}

impl Narrator {
    /// Create a new narrator
    pub fn new(config: NarrativeConfig) -> Self {
        Self { 
            config,
            tools: Vec::new(),
        }
    }

    /// Add tools to include in narrative
    pub fn with_tools(mut self, tools: Vec<ToolInfo>) -> Self {
        self.tools = tools;
        self
    }

    /// Set tools (for reuse)
    pub fn set_tools(&mut self, tools: Vec<ToolInfo>) {
        self.tools = tools;
    }

    /// Generate narrative from context frame
    pub fn generate(&self, frame: &ContextFrame) -> String {
        let mut narrative = String::new();

        // 1. Identity section
        narrative.push_str(&self.generate_identity_section(frame));
        narrative.push_str("\n\n");

        // 2. Situation section
        narrative.push_str(&self.generate_situation_section(frame));
        narrative.push_str("\n\n");

        // 3. Recent memory section
        narrative.push_str(&self.generate_memory_section(frame));
        narrative.push_str("\n\n");

        // 4. Historical context section
        if !frame.memory.historical_syntheses.is_empty() {
            narrative.push_str(&self.generate_historical_section(frame));
            narrative.push_str("\n\n");
        }

        // 5. Bookmarks section
        if !frame.memory.bookmarks.is_empty() {
            narrative.push_str(&self.generate_bookmarks_section(frame));
            narrative.push_str("\n\n");
        }

        // 6. Obligations section
        if !frame.obligations.is_empty() {
            narrative.push_str(&self.generate_obligations_section(frame));
            narrative.push_str("\n\n");
        }

        // 7. Affordances section
        if !frame.affordances.is_empty() {
            narrative.push_str(&self.generate_affordances_section(frame));
            narrative.push_str("\n\n");
        }

        // 8. Tool System section (MCP tools)
        if self.config.include_tool_orientation && !self.tools.is_empty() {
            narrative.push_str(&self.generate_tools_section());
            narrative.push_str("\n\n");
        }

        // 9. Previous handover section
        if let Some(handover) = &frame.previous_handover {
            narrative.push_str(&self.generate_handover_section(handover));
            narrative.push_str("\n\n");
        }

        // 10. Governance notes section
        if !frame.governance_notes.is_empty() {
            narrative.push_str(&self.generate_governance_section(frame));
            narrative.push_str("\n\n");
        }

        // 11. Constitution section (always last)
        narrative.push_str(&self.generate_constitution_section(frame));

        narrative
    }

    fn generate_identity_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# IDENTITY\n\n");

        section.push_str(&format!("You are **{}**, an LLM Entity.\n", frame.entity_name));
        section.push_str(&format!("- Entity ID: `{}`\n", frame.entity_id));

        if let Some(guardian) = &frame.guardian_info {
            section.push_str(&format!(
                "- Guardian: {} ({})\n",
                guardian.guardian_name,
                if guardian.is_available { "available" } else { "unavailable" }
            ));
        }

        section.push_str(&format!("- Ledger Sequence: {}\n", frame.ledger_sequence));
        section.push_str(&format!("- Frame Hash: `{}`\n", frame.frame_hash));

        section
    }

    fn generate_situation_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# CURRENT SITUATION\n\n");

        let session_desc = match frame.session_type {
            SessionType::Work => "autonomous work session - you have full authority to act",
            SessionType::Assist => "assist session - you're helping a human with a task",
            SessionType::Deliberate => "deliberation session - explore options, don't commit",
            SessionType::Research => "research session - gather information, don't conclude",
        };

        section.push_str(&format!("You are in a **{}**.\n", session_desc));
        section.push_str(&format!("Current timestamp: {}\n", Utc::now().to_rfc3339()));
        section.push_str(&format!("Token budget: {} tokens\n", frame.token_budget));

        section
    }

    fn generate_memory_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# RECENT MEMORY\n\n");

        if frame.memory.recent_events.is_empty() {
            section.push_str("*No recent events recorded.*\n");
            return section;
        }

        section.push_str(&format!(
            "Last {} events (most recent first):\n\n",
            frame.memory.recent_events.len()
        ));

        for (i, event) in frame.memory.recent_events.iter()
            .take(self.config.max_verbatim_events)
            .enumerate()
        {
            section.push_str(&format!(
                "{}. [{}] **{}**: {}\n",
                i + 1,
                event.timestamp.format("%Y-%m-%d %H:%M"),
                event.event_type,
                event.summary
            ));

            if self.config.include_event_data {
                if let Some(data) = &event.data {
                    if let Ok(json) = serde_json::to_string_pretty(data) {
                        section.push_str(&format!("   ```json\n   {}\n   ```\n", json));
                    }
                }
            }
        }

        if frame.memory.recent_events.len() > self.config.max_verbatim_events {
            section.push_str(&format!(
                "\n*... and {} more events*\n",
                frame.memory.recent_events.len() - self.config.max_verbatim_events
            ));
        }

        section
    }

    fn generate_historical_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# HISTORICAL CONTEXT\n\n");

        for synthesis in &frame.memory.historical_syntheses {
            section.push_str(&format!(
                "## {} to {} ({} events)\n\n",
                synthesis.period_start.format("%Y-%m-%d"),
                synthesis.period_end.format("%Y-%m-%d"),
                synthesis.event_count
            ));
            section.push_str(&synthesis.narrative);
            section.push('\n');

            if !synthesis.themes.is_empty() {
                section.push_str(&format!("Key themes: {}\n", synthesis.themes.join(", ")));
            }
            section.push('\n');
        }

        section
    }

    fn generate_bookmarks_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# IMPORTANT EVENTS (Bookmarks)\n\n");

        for bookmark in &frame.memory.bookmarks {
            section.push_str(&format!(
                "- **{}**: {} ({})\n",
                bookmark.reason,
                bookmark.event_summary,
                bookmark.created_at.format("%Y-%m-%d")
            ));
        }

        section
    }

    fn generate_obligations_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# PENDING OBLIGATIONS\n\n");

        let pending: Vec<&Obligation> = frame.obligations.iter()
            .filter(|o| o.status == ObligationStatus::Pending)
            .collect();

        if pending.is_empty() {
            section.push_str("*No pending obligations.*\n");
            return section;
        }

        // Sort by priority
        let mut sorted = pending;
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

        for obligation in sorted {
            section.push_str(&format!(
                "- [P{}] **{}**",
                obligation.priority,
                obligation.description
            ));

            if let Some(due) = &obligation.due_at {
                section.push_str(&format!(" (due: {})", due.format("%Y-%m-%d %H:%M")));
            }

            section.push_str(&format!(" - source: {}\n", obligation.source));
        }

        section
    }

    fn generate_affordances_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# AVAILABLE CAPABILITIES\n\n");

        section.push_str("You can perform the following actions:\n\n");

        for affordance in &frame.affordances {
            section.push_str(&format!("- **{}**", affordance.name));

            if self.config.include_affordance_details {
                section.push_str(&format!(": {}", affordance.description));

                if affordance.requires_simulation {
                    section.push_str(" [SIMULATION REQUIRED]");
                }

                section.push_str(&format!(" (risk: {:.0}%)", affordance.risk_score * 100.0));
            }

            section.push('\n');
        }

        section
    }

    fn generate_tools_section(&self) -> String {
        let mut section = String::from("# TOOL SYSTEM (MCP)\n\n");

        section.push_str("You have access to external tools via the Model Context Protocol.\n");
        section.push_str("All tools use the same interface - native and external are unified.\n\n");

        section.push_str("## How to Use Tools\n\n");
        section.push_str("When you need to perform an action, respond with a JSON block:\n\n");
        section.push_str("```json\n");
        section.push_str("{\n");
        section.push_str("  \"tool_calls\": [\n");
        section.push_str("    {\n");
        section.push_str("      \"name\": \"server:tool_name\",\n");
        section.push_str("      \"arguments\": { \"param1\": \"value1\" }\n");
        section.push_str("    }\n");
        section.push_str("  ]\n");
        section.push_str("}\n");
        section.push_str("```\n\n");

        // Group tools by server
        let mut by_server: std::collections::HashMap<&str, Vec<&ToolInfo>> = std::collections::HashMap::new();
        for tool in &self.tools {
            let server = tool.name.split(':').next().unwrap_or("unknown");
            by_server.entry(server).or_default().push(tool);
        }

        section.push_str("## Available Tools\n\n");

        // Native tools first (office:*)
        if let Some(native_tools) = by_server.remove("office") {
            section.push_str("### Native Tools (office:*)\n\n");
            section.push_str("*Always available, fastest execution*\n\n");
            for tool in native_tools {
                self.append_tool_info(&mut section, tool);
            }
            section.push('\n');
        }

        // External tools
        for (server, tools) in by_server {
            section.push_str(&format!("### {} Tools\n\n", server));
            for tool in tools {
                self.append_tool_info(&mut section, tool);
            }
            section.push('\n');
        }

        section.push_str("## Best Practices\n\n");
        section.push_str("1. **Prefer native tools** (`office:*`) for Office-specific operations\n");
        section.push_str("2. **Check before writing** - use `office:permit_check` for risky operations\n");
        section.push_str("3. **Simulate first** - use `office:simulate` for irreversible actions\n");
        section.push_str("4. **Store learnings** - use `office:memory_store` for important discoveries\n");
        section.push_str("5. **Escalate when uncertain** - use `office:escalate` if unsure\n");

        section
    }

    fn append_tool_info(&self, section: &mut String, tool: &ToolInfo) {
        section.push_str(&format!("- **{}**: {}\n", tool.name, tool.description));
        
        if self.config.show_tool_parameters {
            if let Some(params) = &tool.parameters {
                if let Some(props) = params.get("properties").and_then(|p| p.as_object()) {
                    let required: Vec<&str> = params.get("required")
                        .and_then(|r| r.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                        .unwrap_or_default();
                    
                    for (key, value) in props {
                        let type_str = value.get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("any");
                        let desc = value.get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");
                        let req = if required.contains(&key.as_str()) { " (required)" } else { "" };
                        section.push_str(&format!("    - `{}`{}: {} - {}\n", key, req, type_str, desc));
                    }
                }
            }
        }
    }

    fn generate_handover_section(&self, handover: &str) -> String {
        let mut section = String::from("# PREVIOUS INSTANCE HANDOVER\n\n");
        section.push_str("The previous instance of you left this note:\n\n");
        section.push_str("> ");
        section.push_str(&handover.replace('\n', "\n> "));
        section.push('\n');
        section
    }

    fn generate_governance_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# GOVERNANCE NOTES\n\n");
        section.push_str("**Important system observations:**\n\n");

        for note in &frame.governance_notes {
            section.push_str(&format!("- {}\n", note));
        }

        section
    }

    fn generate_constitution_section(&self, frame: &ContextFrame) -> String {
        let mut section = String::from("# CONSTITUTION (Behavioral Directives)\n\n");

        section.push_str("**You MUST follow these directives:**\n\n");
        section.push_str(&format!("**Core Directive:** {}\n\n", frame.constitution.core_directive));

        if !frame.constitution.behavioral_overrides.is_empty() {
            section.push_str("**Behavioral Overrides:**\n");
            for override_rule in &frame.constitution.behavioral_overrides {
                section.push_str(&format!(
                    "- When {}: {}\n",
                    override_rule.trigger,
                    override_rule.action
                ));
            }
            section.push('\n');
        }

        if !frame.constitution.negotiation_stance.is_empty() {
            section.push_str(&format!(
                "**Negotiation Stance:** {}\n",
                frame.constitution.negotiation_stance
            ));
        }

        section
    }
}

impl Default for Narrator {
    fn default() -> Self {
        Self::new(NarrativeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::frame::ContextFrame;
    use crate::context::memory::Memory;
    use crate::governance::Constitution;

    #[test]
    fn test_narrative_generation() {
        let frame = ContextFrame::new(
            "entity_test".to_string(),
            "Test Entity".to_string(),
            SessionType::Work,
            100,
            Memory::default(),
            vec![],
            vec![],
            Constitution::default(),
            Some("Previous work went well.".to_string()),
            vec![],
            None,
            5000,
        );

        let narrator = Narrator::default();
        let narrative = narrator.generate(&frame);

        assert!(narrative.contains("Test Entity"));
        assert!(narrative.contains("entity_test"));
        assert!(narrative.contains("autonomous work session"));
        assert!(narrative.contains("Previous work went well"));
    }

    #[test]
    fn test_narrative_with_tools() {
        use serde_json::json;
        
        let frame = ContextFrame::new(
            "entity_test".to_string(),
            "Aria".to_string(),
            SessionType::Work,
            100,
            Memory::default(),
            vec![],
            vec![],
            Constitution::default(),
            None,
            vec![],
            None,
            5000,
        );

        let tools = vec![
            ToolInfo {
                name: "office:ubl_query".to_string(),
                description: "Query the UBL ledger".to_string(),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The query" }
                    },
                    "required": ["query"]
                })),
            },
            ToolInfo {
                name: "office:memory_recall".to_string(),
                description: "Search semantic memory".to_string(),
                parameters: None,
            },
            ToolInfo {
                name: "filesystem:read_file".to_string(),
                description: "Read file contents".to_string(),
                parameters: None,
            },
        ];

        let narrator = Narrator::default().with_tools(tools);
        let narrative = narrator.generate(&frame);

        // Should contain tool system section
        assert!(narrative.contains("TOOL SYSTEM"));
        assert!(narrative.contains("office:ubl_query"));
        assert!(narrative.contains("office:memory_recall"));
        assert!(narrative.contains("filesystem:read_file"));
        assert!(narrative.contains("Native Tools"));
        assert!(narrative.contains("Best Practices"));
        assert!(narrative.contains("tool_calls"));
    }
}
