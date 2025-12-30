//! Task Executor
//!
//! Executes tasks for UBL Messenger, managing LLM sessions, progress streaming,
//! and the approval/acceptance workflow.
//!
//! Key differences from JobExecutor:
//! 1. Tasks require explicit human acceptance after completion
//! 2. Tasks support git versioning for document output
//! 3. Tasks have bidirectional approval (human↔agent can both create drafts)
//!
//! Flow:
//! 1. Task is created (draft)
//! 2. Other party approves
//! 3. If agent is assigned, execute with Entity/Instance
//! 4. Progress is streamed via SSE
//! 5. On completion, human must accept
//! 6. Accepted tasks are versioned in git

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::entity::{Entity, EntityId, EntityParams, EntityType, EntityRepository};
use crate::context::{ContextFrameBuilder, Narrator, NarrativeConfig};
use crate::session::SessionType;
use crate::ubl_client::UblClient;
use crate::llm::{LlmMessage, LlmRequest, SmartRouter, TaskType, RoutingPreferences};
use crate::governance::Constitution;
use crate::{OfficeError, Result};

use super::types::{
    Task, TaskId, TaskResult, TaskArtifact, ArtifactType,
    TaskProgressUpdate, TaskLogEntry, LogLevel, GitCommit,
    TaskCreatedEvent, TaskApprovedEvent, TaskStartedEvent,
    TaskProgressEvent, TaskCompletedEvent, TaskAcceptedEvent,
};
use super::fsm::{TaskState, TaskStateTracker, TransitionReason};
use super::cards::{
    TaskCardBuilder, TaskCreationCard, TaskProgressCard, TaskCompletedCard,
    TaskActor, ActorType, TaskDetails, TaskProgressInfo, TaskOutcome, TaskStats,
    GitCommitInfo,
};

/// Progress update types for SSE streaming
#[derive(Debug, Clone)]
pub enum TaskProgressMessage {
    /// Status update with progress percentage
    Progress(TaskProgressUpdate),
    /// Log entry
    Log(TaskLogEntry),
    /// Task completed
    Completed(TaskResult),
    /// Task failed
    Failed(String),
}

/// Task Executor - Executes tasks using LLM entities
pub struct TaskExecutor {
    ubl_client: Arc<UblClient>,
    entity_repository: Arc<EntityRepository>,
    router: Arc<SmartRouter>,
    container_id: String,
}

impl TaskExecutor {
    /// Create a new task executor
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

    /// Execute a task (synchronous, for simple tasks)
    pub async fn execute_task(&self, task: &mut Task) -> Result<TaskResult> {
        let start_time = Utc::now();

        // 1. Validate task state
        if task.status != super::types::TaskStatus::Approved {
            return Err(OfficeError::JobTransitionError(
                "Task must be approved before execution".to_string(),
            ));
        }

        // 2. Start the task
        task.start();
        self.publish_started_event(task).await?;

        // 3. Get or create the Entity (The Chair)
        let entity = self.get_or_create_agent_entity(&task.assigned_to).await?;

        // 4. Build context frame
        let context = ContextFrameBuilder::new(
            entity.clone(),
            SessionType::Work,
            self.ubl_client.clone(),
        )
        .build()
        .await?;

        // 5. Generate the Narrative
        let narrator = Narrator::new(NarrativeConfig::default());
        let base_narrative = narrator.generate(&context);

        let narrative = format!(
            "{}\n\n## Task\n\n**Title:** {}\n**Description:** {}\n**Priority:** {:?}\n**Deadline:** {}\n**Estimated Cost:** {}\n\n## Your Response\n\nComplete this task thoroughly. Provide your response with clear structure.",
            base_narrative,
            task.title,
            task.description.as_deref().unwrap_or("No description provided"),
            task.priority,
            task.deadline.map(|d| d.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
            task.estimated_cost.as_deref().unwrap_or("Not specified")
        );

        // 6. Determine task type for smart routing
        let task_type = self.classify_task(task);
        let routing_prefs = RoutingPreferences::default();

        // 7. Execute the LLM call
        let request = LlmRequest::new(vec![LlmMessage::user(&narrative)])
            .with_system("You are a helpful AI assistant. Complete the task described in the user message. Be thorough and well-organized.")
            .with_max_tokens(4096)
            .with_temperature(0.7);

        let response = self.router.route(request, task_type, &routing_prefs).await?;

        // 8. Build result
        let duration = (Utc::now() - start_time).num_seconds() as u64;

        // Create artifact from response
        let artifact = TaskArtifact {
            id: format!("artifact_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string()),
            name: format!("{} - Result", task.title),
            artifact_type: ArtifactType::Document,
            url: String::new(), // Would be set after storage
            size_bytes: Some(response.content.len() as u64),
            mime_type: Some("text/markdown".to_string()),
            created_at: Utc::now(),
        };

        let result = TaskResult {
            task_id: task.id.clone(),
            success: true,
            summary: self.extract_summary(&response.content),
            artifacts: vec![artifact],
            duration_seconds: duration,
            tokens_used: Some(response.usage.total_tokens as u64),
            git_commit: None, // Git commit happens at acceptance
            error: None,
        };

        // 9. Complete the task
        task.complete(&result);

        // 10. Record the session
        self.entity_repository.record_session(
            &entity.id,
            &format!("task_{}", task.id),
            response.usage.total_tokens as u64,
            Some(self.generate_handover(task, &response.content)),
        ).await?;

        // 11. Publish completion event
        self.publish_completed_event(task, &result).await?;

        Ok(result)
    }

    /// Execute task with progress streaming (for complex tasks)
    pub async fn execute_with_progress(
        &self,
        task: Task,
    ) -> Result<(mpsc::Receiver<TaskProgressMessage>, tokio::task::JoinHandle<()>)> {
        let (tx, rx) = mpsc::channel(64);

        // Clone what we need for the async task
        let executor = TaskExecutor {
            ubl_client: self.ubl_client.clone(),
            entity_repository: self.entity_repository.clone(),
            router: self.router.clone(),
            container_id: self.container_id.clone(),
        };

        let mut task = task;
        let task_id = task.id.clone();

        // Spawn execution task
        let handle = tokio::spawn(async move {
            // Send start progress
            let _ = tx.send(TaskProgressMessage::Progress(TaskProgressUpdate {
                task_id: task_id.clone(),
                progress: 0,
                current_step: "Initializing".to_string(),
                message: "Loading entity and context...".to_string(),
                timestamp: Utc::now(),
            })).await;

            let _ = tx.send(TaskProgressMessage::Log(TaskLogEntry {
                task_id: task_id.clone(),
                level: LogLevel::Info,
                message: "Task execution started".to_string(),
                timestamp: Utc::now(),
            })).await;

            // Send progress updates as we go
            let _ = tx.send(TaskProgressMessage::Progress(TaskProgressUpdate {
                task_id: task_id.clone(),
                progress: 20,
                current_step: "Context".to_string(),
                message: "Building context frame...".to_string(),
                timestamp: Utc::now(),
            })).await;

            let _ = tx.send(TaskProgressMessage::Progress(TaskProgressUpdate {
                task_id: task_id.clone(),
                progress: 40,
                current_step: "LLM".to_string(),
                message: "Sending to LLM...".to_string(),
                timestamp: Utc::now(),
            })).await;

            // Execute task
            match executor.execute_task(&mut task).await {
                Ok(result) => {
                    let _ = tx.send(TaskProgressMessage::Progress(TaskProgressUpdate {
                        task_id: task_id.clone(),
                        progress: 100,
                        current_step: "Complete".to_string(),
                        message: "Task completed successfully".to_string(),
                        timestamp: Utc::now(),
                    })).await;

                    let _ = tx.send(TaskProgressMessage::Completed(result)).await;
                }
                Err(e) => {
                    let _ = tx.send(TaskProgressMessage::Log(TaskLogEntry {
                        task_id: task_id.clone(),
                        level: LogLevel::Error,
                        message: format!("Task failed: {}", e),
                        timestamp: Utc::now(),
                    })).await;

                    let _ = tx.send(TaskProgressMessage::Failed(e.to_string())).await;
                }
            }
        });

        Ok((rx, handle))
    }

    /// Create task creation card (for draft phase)
    pub fn create_creation_card(
        &self,
        task: &Task,
        creator_name: &str,
        assignee_name: &str,
    ) -> TaskCreationCard {
        let builder = TaskCardBuilder::new(
            task.id.clone(),
            task.conversation_id.clone(),
            task.tenant_id.clone(),
            TaskActor {
                entity_id: task.created_by.clone(),
                display_name: creator_name.to_string(),
                actor_type: if task.created_by.starts_with("agent_") {
                    ActorType::Agent
                } else {
                    ActorType::Human
                },
            },
            TaskActor {
                entity_id: task.assigned_to.clone(),
                display_name: assignee_name.to_string(),
                actor_type: if task.assigned_to.starts_with("agent_") {
                    ActorType::Agent
                } else {
                    ActorType::Human
                },
            },
        );

        let details = TaskDetails {
            task_id: task.id.clone(),
            title: task.title.clone(),
            description: task.description.clone(),
            priority: task.priority,
            deadline: task.deadline,
            estimated_cost: task.estimated_cost.clone(),
        };

        builder.build_creation(details, task.attachments.clone())
    }

    /// Create progress card (for running phase)
    pub fn create_progress_card(
        &self,
        task: &Task,
        creator_name: &str,
        assignee_name: &str,
    ) -> TaskProgressCard {
        let builder = TaskCardBuilder::new(
            task.id.clone(),
            task.conversation_id.clone(),
            task.tenant_id.clone(),
            TaskActor {
                entity_id: task.created_by.clone(),
                display_name: creator_name.to_string(),
                actor_type: ActorType::Human,
            },
            TaskActor {
                entity_id: task.assigned_to.clone(),
                display_name: assignee_name.to_string(),
                actor_type: ActorType::Agent,
            },
        );

        let progress = TaskProgressInfo {
            percent: task.progress,
            status_line: task.current_step.clone().unwrap_or_else(|| "Working...".to_string()),
            current_step: task.current_step.clone(),
            blockers: None,
            steps: None,
            last_update_at: Utc::now(),
        };

        builder.build_progress(progress)
    }

    /// Create completed card (for acceptance phase)
    pub fn create_completed_card(
        &self,
        task: &Task,
        result: &TaskResult,
        creator_name: &str,
        assignee_name: &str,
    ) -> TaskCompletedCard {
        let builder = TaskCardBuilder::new(
            task.id.clone(),
            task.conversation_id.clone(),
            task.tenant_id.clone(),
            TaskActor {
                entity_id: task.created_by.clone(),
                display_name: creator_name.to_string(),
                actor_type: ActorType::Human,
            },
            TaskActor {
                entity_id: task.assigned_to.clone(),
                display_name: assignee_name.to_string(),
                actor_type: ActorType::Agent,
            },
        );

        let outcome = TaskOutcome {
            success: result.success,
            summary: result.summary.clone(),
            completed_at: task.completed_at.unwrap_or_else(Utc::now),
        };

        let git_commit = result.git_commit.as_ref().map(|gc| GitCommitInfo {
            hash: gc.hash.clone(),
            message: gc.message.clone(),
            branch: gc.branch.clone(),
            repo_url: gc.repo_url.clone(),
        });

        let stats = TaskStats {
            duration_seconds: result.duration_seconds,
            tokens_used: result.tokens_used,
            artifact_count: result.artifacts.len(),
        };

        builder.build_completed(outcome, result.artifacts.clone(), git_commit, stats)
    }

    // ============ Private Helpers ============

    /// Get or create agent entity (The Chair)
    async fn get_or_create_agent_entity(&self, agent_id: &str) -> Result<Entity> {
        let entity_id: EntityId = agent_id.to_string();

        match self.entity_repository.get(&entity_id).await {
            Ok(entity) => Ok(entity),
            Err(OfficeError::EntityNotFound(_)) => {
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
    fn classify_task(&self, task: &Task) -> TaskType {
        let title_lower = task.title.to_lowercase();
        let desc_lower = task.description.as_deref().unwrap_or("").to_lowercase();
        let combined = format!("{} {}", title_lower, desc_lower);

        if combined.contains("code") || combined.contains("debug")
            || combined.contains("fix") || combined.contains("implement")
            || combined.contains("refactor")
        {
            TaskType::Coding
        } else if combined.contains("write") || combined.contains("document")
            || combined.contains("explain") || combined.contains("report")
        {
            TaskType::Writing
        } else if combined.contains("analyze") || combined.contains("review")
            || combined.contains("audit")
        {
            TaskType::Analysis
        } else if combined.contains("brainstorm") || combined.contains("creative")
            || combined.contains("design")
        {
            TaskType::Creative
        } else if combined.contains("complex") || combined.contains("plan")
            || combined.contains("architect")
        {
            TaskType::Complex
        } else if combined.len() < 50 {
            TaskType::Quick
        } else {
            TaskType::Unknown
        }
    }

    /// Extract a summary from the response
    fn extract_summary(&self, content: &str) -> String {
        let summary: String = content.chars().take(200).collect();
        if content.len() > 200 {
            format!("{}...", summary)
        } else {
            summary
        }
    }

    /// Generate handover for the next instance
    fn generate_handover(&self, task: &Task, response: &str) -> String {
        format!(
            "## Session Handover\n\n**Completed Task:** {}\n\n**Summary:** {}\n\n**Timestamp:** {}\n\nThis handover preserves context for the next instance sitting in this Chair.",
            task.title,
            self.extract_summary(response),
            Utc::now().to_rfc3339()
        )
    }

    /// Default baseline for new entities
    fn default_baseline(&self) -> String {
        "You are a professional AI assistant working in the OFFICE environment. \
         You complete tasks thoroughly and communicate clearly. \
         All your work must be approved by humans before becoming official.".to_string()
    }

    // ============ UBL Event Publishing ============

    /// Publish task.started event to UBL
    async fn publish_started_event(&self, task: &Task) -> Result<()> {
        let event = TaskStartedEvent {
            task_id: task.id.clone(),
            timestamp: Utc::now(),
        };

        self.commit_event(&event, "task.started").await
    }

    /// Publish task.completed event to UBL
    async fn publish_completed_event(&self, task: &Task, result: &TaskResult) -> Result<()> {
        let event = TaskCompletedEvent {
            task_id: task.id.clone(),
            success: result.success,
            summary: result.summary.clone(),
            artifact_count: result.artifacts.len(),
            duration_seconds: result.duration_seconds,
            git_commit_hash: result.git_commit.as_ref().map(|gc| gc.hash.clone()),
            timestamp: Utc::now(),
        };

        self.commit_event(&event, "task.completed").await
    }

    /// Commit an event to UBL
    async fn commit_event<T: serde::Serialize>(&self, event: &T, event_type: &str) -> Result<()> {
        let event_json = serde_json::to_value(event)
            .map_err(|e| OfficeError::UblError(format!("Serialize failed: {}", e)))?;

        // Canonicalize and hash
        let canonical = ubl_atom::canonicalize(&event_json)
            .map_err(|e| OfficeError::UblError(format!("Canonicalize failed: {}", e)))?;
        let atom_hash = ubl_kernel::hash_atom(&canonical);

        // Get current state
        let state = self.ubl_client.get_state(&self.container_id).await?;

        // Build signing bytes
        let signing_data = format!(
            "{}|{}|{}|{}|{}|observation|0",
            1, // version
            self.container_id,
            state.sequence + 1,
            state.last_hash,
            atom_hash
        );

        // Sign with the UBL client's key
        let signature = self.ubl_client.sign(signing_data.as_bytes());
        let author_pubkey = self.ubl_client.pubkey_hex().to_string();

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
            author_pubkey,
            signature: format!("ed25519:{}", signature),
        };

        // Commit
        self.ubl_client.commit(link).await?;

        tracing::info!("Committed {} event for task", event_type);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_classification() {
        // Would need mocks for full testing
        // Placeholder for test structure
    }
}
