//! Job Executor
//!
//! Executes jobs for UBL Messenger, managing LLM sessions, progress streaming,
//! and approval workflows.
//!
//! This is where the magic happens:
//! 1. Entity (Chair) is loaded/created from UBL
//! 2. Narrative is composed with context
//! 3. LLM instance receives beautiful onboarding
//! 4. Work is done, progress streamed
//! 5. Handover is written for the next instance

use std::sync::Arc;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::{Stream, StreamExt};
use futures::pin_mut;
use chrono::Utc;
use tokio::sync::mpsc;

use crate::entity::{Entity, EntityId, EntityParams, EntityType, EntityRepository};
use crate::context::{ContextFrameBuilder, Narrator, NarrativeConfig};
use crate::session::{Session, SessionType, SessionMode};
use crate::ubl_client::UblClient;
use crate::llm::{LlmMessage, LlmRequest, SmartRouter, TaskType, RoutingPreferences};
use crate::governance::Constitution;
use crate::{OfficeError, Result};

use super::types::{
    Job, JobId, JobResult, JobStep, StepStatus,
    ApprovalRequest, ApprovalDecision, ConversationContext, ProgressUpdate,
};
use super::conversation_context::ConversationContextBuilder;

/// Job Executor - Executes jobs using LLM entities
pub struct JobExecutor {
    ubl_client: Arc<UblClient>,
    entity_repository: Arc<EntityRepository>,
    router: Arc<SmartRouter>,
    container_id: String,
}

impl JobExecutor {
    /// Create a new job executor
    pub fn new(
        ubl_client: Arc<UblClient>,
        entity_repository: Arc<EntityRepository>,
        router: Arc<SmartRouter>,
        container_id: &str,
    ) -> Self {
        Self {
            ubl_client,
            entity_repository,
            router,
            container_id: container_id.to_string(),
        }
    }

    /// Execute a job
    ///
    /// This is the main entry point for job execution.
    /// The Entity (Chair) is permanent, the LLM instance is ephemeral.
    /// Each instance receives beautiful onboarding from its Chair.
    pub async fn execute_job(
        &self,
        job: Job,
        conversation_context: ConversationContext,
    ) -> Result<JobResult> {
        let start_time = Utc::now();
        
        // 1. Get or create the Entity (The Chair)
        let entity = self.get_or_create_agent_entity(&job.assigned_to).await?;
        
        // 2. Build context frame from UBL
        let context = ContextFrameBuilder::new(
            entity.clone(),
            SessionType::Work,
            self.ubl_client.clone(),
        )
        .build()
        .await?;
        
        // 3. Generate the Narrative - The onboarding for this ephemeral instance
        let narrator = Narrator::new(NarrativeConfig::default());
        let base_narrative = narrator.generate(&context);
        
        // Build the full narrative with job context
        let narrative = format!(
            "{}\n\n## Current Task\n\n**Job Title:** {}\n**Description:** {}\n**Conversation:** {}\n\n{}\n\n## Your Response\n\nProvide your response to complete this task.",
            base_narrative,
            job.title,
            job.description.as_deref().unwrap_or("No description provided"),
            conversation_context.conversation_id,
            ConversationContextBuilder::new(conversation_context.conversation_id.clone())
                .with_participants(conversation_context.participants.clone())
                .with_recent_messages(conversation_context.recent_messages.clone())
                .with_active_jobs(conversation_context.active_jobs.clone())
                .to_narrative()
        );
        
        // 4. Determine task type for smart routing
        let task_type = self.classify_task(&job);
        let routing_prefs = RoutingPreferences::default();
        
        // 5. Execute the LLM call
        let request = LlmRequest::new(vec![LlmMessage::user(&narrative)])
            .with_system("You are a helpful AI assistant. Complete the task described in the user message.")
            .with_max_tokens(4096)
            .with_temperature(0.7);
        
        let response = self.router.route(request, task_type, &routing_prefs).await?;
        
        // 6. Build result
        let duration = (Utc::now() - start_time).num_seconds() as u64;
        
        let result = JobResult {
            job_id: job.id.clone(),
            success: true,
            summary: Some(self.extract_summary(&response.content)),
            output: Some(response.content.clone()),
            artifacts: vec![],
            tokens_used: response.usage.total_tokens as u64,
            value_created: None,
            duration_seconds: duration,
            error: None,
        };
        
        // 7. Record the session for this Entity
        self.entity_repository.record_session(
            &entity.id,
            &format!("job_{}", job.id),
            response.usage.total_tokens as u64,
            Some(self.generate_handover(&job, &response.content)),
        ).await?;
        
        // 8. Publish completion event to UBL
        self.publish_completion_event(&job, &result).await?;
        
        Ok(result)
    }

    /// Get or create agent entity (The Chair)
    async fn get_or_create_agent_entity(&self, agent_id: &str) -> Result<Entity> {
        let entity_id: EntityId = agent_id.to_string();
        
        // Try to get existing entity
        match self.entity_repository.get(&entity_id).await {
            Ok(entity) => Ok(entity),
            Err(OfficeError::EntityNotFound(_)) => {
                // Create new entity
                let params = EntityParams {
                    name: format!("Agent {}", agent_id),
                    entity_type: EntityType::Autonomous,
                    guardian_id: None,
                    constitution: Some(Constitution::default()),
                    baseline_narrative: Some(self.default_baseline()),
                    metadata: None,
                };
                
                self.entity_repository.get_or_create(&entity_id, params).await
            }
            Err(e) => Err(e),
        }
    }

    /// Classify task for routing
    fn classify_task(&self, job: &Job) -> TaskType {
        let title_lower = job.title.to_lowercase();
        let desc_lower = job.description.as_deref().unwrap_or("").to_lowercase();
        let combined = format!("{} {}", title_lower, desc_lower);
        
        if combined.contains("code") || combined.contains("debug") || 
           combined.contains("fix") || combined.contains("implement") ||
           combined.contains("refactor") {
            TaskType::Coding
        } else if combined.contains("write") || combined.contains("document") ||
                  combined.contains("explain") {
            TaskType::Writing
        } else if combined.contains("analyze") || combined.contains("review") ||
                  combined.contains("audit") {
            TaskType::Analysis
        } else if combined.contains("brainstorm") || combined.contains("creative") ||
                  combined.contains("design") {
            TaskType::Creative
        } else if combined.contains("complex") || combined.contains("plan") ||
                  combined.contains("architect") {
            TaskType::Complex
        } else if combined.len() < 50 {
            TaskType::Quick
        } else {
            TaskType::Unknown
        }
    }

    /// Extract a summary from the response
    fn extract_summary(&self, content: &str) -> String {
        // Take first 200 chars as summary
        let summary: String = content.chars().take(200).collect();
        if content.len() > 200 {
            format!("{}...", summary)
        } else {
            summary
        }
    }

    /// Generate handover for the next instance
    fn generate_handover(&self, job: &Job, response: &str) -> String {
        format!(
            "## Session Handover\n\n**Completed Task:** {}\n\n**Summary:** {}\n\n**Timestamp:** {}\n\nThis handover preserves context for the next instance sitting in this Chair.",
            job.title,
            self.extract_summary(response),
            Utc::now().to_rfc3339()
        )
    }

    /// Default baseline for new entities
    fn default_baseline(&self) -> String {
        "You are a professional AI assistant working in the OFFICE environment. \
         You have access to tools and can complete tasks efficiently. \
         You value accuracy, helpfulness, and clear communication.".to_string()
    }

    /// Publish job completion event to UBL
    async fn publish_completion_event(&self, job: &Job, result: &JobResult) -> Result<()> {
        // Build event
        let event = serde_json::json!({
            "type": "job.completed",
            "job_id": job.id,
            "success": result.success,
            "tokens_used": result.tokens_used,
            "duration_seconds": result.duration_seconds,
            "timestamp": Utc::now().to_rfc3339()
        });
        
        // Canonicalize and hash
        let canonical = ubl_atom::canonicalize(&event)
            .map_err(|e| OfficeError::UblError(format!("Canonicalize failed: {}", e)))?;
        let atom_hash = ubl_kernel::hash_atom(&canonical);
        
        // Get current state
        let state = self.ubl_client.get_state(&self.container_id).await?;
        
        // Build link commit
        let link = crate::ubl_client::LinkCommit {
            version: 1,
            container_id: self.container_id.clone(),
            expected_sequence: state.sequence + 1,
            previous_hash: state.last_hash,
            atom_hash,
            intent_class: "observation".to_string(),
            physics_delta: 0,
            pact: None,
            author_pubkey: "office".to_string(), // TODO: Use actual key
            signature: "mock".to_string(), // TODO: Sign properly
        };
        
        // Commit
        self.ubl_client.commit(link).await?;
        
        Ok(())
    }

    /// Execute job with progress streaming (for complex jobs)
    pub async fn execute_with_progress(
        &self,
        job: Job,
        conversation_context: ConversationContext,
    ) -> Result<impl Stream<Item = ProgressUpdate>> {
        let (tx, rx) = mpsc::channel(32);
        
        // Clone what we need for the async task
        let executor = JobExecutor {
            ubl_client: self.ubl_client.clone(),
            entity_repository: self.entity_repository.clone(),
            router: self.router.clone(),
            container_id: self.container_id.clone(),
        };
        
        let job_id = job.id.clone();
        
        // Spawn execution task
        tokio::spawn(async move {
            // Send start progress
            let _ = tx.send(ProgressUpdate::StepCompleted(JobStep {
                id: "start".to_string(),
                description: "Starting job execution - Loading entity and context...".to_string(),
                status: StepStatus::InProgress,
            })).await;
            
            // Execute job
            match executor.execute_job(job, conversation_context).await {
                Ok(result) => {
                    let _ = tx.send(ProgressUpdate::Completed(result)).await;
                }
                Err(e) => {
                    let _ = tx.send(ProgressUpdate::Failed(e.to_string())).await;
                }
            }
        });
        
        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_classification() {
        // We'd need to mock dependencies for full testing
        // This is a placeholder for the test structure
    }
}
