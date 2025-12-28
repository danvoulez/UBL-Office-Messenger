//! Conversation Context Builder
//!
//! Builds conversation context for job execution.

use super::types::{ConversationContext, Message, Job};
use crate::Result;

/// Builder for conversation context
pub struct ConversationContextBuilder {
    conversation_id: String,
    participants: Vec<String>,
    recent_messages: Vec<Message>,
    active_jobs: Vec<Job>,
    recent_events: Vec<String>,
}

impl ConversationContextBuilder {
    /// Create a new builder
    pub fn new(conversation_id: String) -> Self {
        Self {
            conversation_id,
            participants: vec![],
            recent_messages: vec![],
            active_jobs: vec![],
            recent_events: vec![],
        }
    }

    /// Add participants
    pub fn with_participants(mut self, participants: Vec<String>) -> Self {
        self.participants = participants;
        self
    }

    /// Add recent messages
    pub fn with_recent_messages(mut self, messages: Vec<Message>) -> Self {
        self.recent_messages = messages;
        self
    }

    /// Add active jobs
    pub fn with_active_jobs(mut self, jobs: Vec<Job>) -> Self {
        self.active_jobs = jobs;
        self
    }

    /// Add recent events
    pub fn with_recent_events(mut self, events: Vec<String>) -> Self {
        self.recent_events = events;
        self
    }

    /// Build conversation context
    pub fn build(self) -> ConversationContext {
        ConversationContext {
            conversation_id: self.conversation_id,
            participants: self.participants,
            recent_messages: self.recent_messages,
            active_jobs: self.active_jobs,
            recent_events: self.recent_events,
        }
    }

    /// Get recent messages (last N)
    pub fn recent_messages(&self, limit: usize) -> Vec<Message> {
        let start = self.recent_messages.len().saturating_sub(limit);
        self.recent_messages[start..].to_vec()
    }

    /// Generate narrative from context
    pub fn to_narrative(&self) -> String {
        format!(
            r#"
CONTEXTO DA CONVERSA

Participantes:
{}

Últimas 10 mensagens:
{}

Jobs ativos:
{}

Você deve:
- Manter o tom profissional mas amigável
- Usar cards quando apropriado
- Pedir aprovação para ações importantes
- Nunca inventar informações
"#,
            self.participants_narrative(),
            self.messages_narrative(),
            self.active_jobs_narrative(),
        )
    }

    fn participants_narrative(&self) -> String {
        self.participants
            .iter()
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn messages_narrative(&self) -> String {
        self.recent_messages(10)
            .iter()
            .map(|m| format!("{} ({}): {}", m.from, m.timestamp.format("%H:%M"), m.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn active_jobs_narrative(&self) -> String {
        if self.active_jobs.is_empty() {
            "Nenhum job ativo".to_string()
        } else {
            self.active_jobs
                .iter()
                .map(|j| format!("- {} ({:?})", j.title, j.status))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

