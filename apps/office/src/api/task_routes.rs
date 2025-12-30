//! Task API Routes
//!
//! HTTP endpoints for the task formalization system.
//!
//! Endpoints:
//! - POST /tasks - Create a new task draft
//! - GET /tasks/:id - Get task by ID
//! - POST /tasks/:id/approve - Approve a task draft
//! - POST /tasks/:id/reject - Reject a task draft
//! - POST /tasks/:id/execute - Start task execution
//! - GET /tasks/:id/stream - SSE stream for task progress
//! - POST /tasks/:id/accept - Accept completed task
//! - POST /tasks/:id/dispute - Dispute completed task
//! - POST /tasks/:id/cancel - Cancel task

use std::sync::Arc;
use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::{error, info};

use crate::task::{
    Task, TaskId, TaskStatus,
    CreateTaskRequest, ApproveTaskRequest, AcceptTaskRequest,
    RejectTaskRequest, DisputeTaskRequest,
    TaskExecutor, TaskProgressMessage,
    TaskCreationCard, TaskProgressCard, TaskCompletedCard, TaskCard,
};
use crate::ubl_client::UblClient;
use crate::entity::EntityRepository;
use crate::llm::SmartRouter;
use crate::{OfficeError, Result};

/// Task-specific state
pub struct TaskState {
    pub ubl_client: Arc<UblClient>,
    pub entity_repository: Arc<EntityRepository>,
    pub router: Arc<SmartRouter>,
    pub container_id: String,
    /// In-memory task store (would be replaced by UBL projection in production)
    pub tasks: RwLock<HashMap<TaskId, Task>>,
}

impl TaskState {
    pub fn new(
        ubl_client: Arc<UblClient>,
        entity_repository: Arc<EntityRepository>,
        router: Arc<SmartRouter>,
        container_id: String,
    ) -> Self {
        Self {
            ubl_client,
            entity_repository,
            router,
            container_id,
            tasks: RwLock::new(HashMap::new()),
        }
    }

    fn executor(&self) -> TaskExecutor {
        TaskExecutor::new(
            self.ubl_client.clone(),
            self.entity_repository.clone(),
            self.router.clone(),
            &self.container_id,
        )
    }
}

pub type SharedTaskState = Arc<TaskState>;

/// Create the task router
pub fn task_router(state: SharedTaskState) -> Router {
    Router::new()
        // CRUD
        .route("/tasks", post(create_task))
        .route("/tasks/:id", get(get_task))
        
        // Lifecycle
        .route("/tasks/:id/approve", post(approve_task))
        .route("/tasks/:id/reject", post(reject_task))
        .route("/tasks/:id/execute", post(execute_task))
        .route("/tasks/:id/stream", get(execute_task_stream))
        .route("/tasks/:id/accept", post(accept_task))
        .route("/tasks/:id/dispute", post(dispute_task))
        .route("/tasks/:id/cancel", post(cancel_task))
        
        // Cards
        .route("/tasks/:id/card", get(get_task_card))
        
        .with_state(state)
}

// ============ Responses ============

#[derive(Debug, Serialize)]
struct TaskResponse {
    task: Task,
    card: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct TaskActionResponse {
    success: bool,
    task: Task,
    card: Option<serde_json::Value>,
    event_id: Option<String>,
}

#[derive(Debug)]
enum TaskError {
    NotFound(String),
    BadRequest(String),
    InvalidState(String),
    Internal(String),
}

impl IntoResponse for TaskError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            TaskError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            TaskError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            TaskError::InvalidState(msg) => (StatusCode::CONFLICT, msg),
            TaskError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

impl From<OfficeError> for TaskError {
    fn from(err: OfficeError) -> Self {
        match err {
            OfficeError::JobTransitionError(msg) => TaskError::InvalidState(msg),
            _ => TaskError::Internal(err.to_string()),
        }
    }
}

// ============ Handlers ============

/// POST /tasks - Create a new task draft
async fn create_task(
    State(state): State<SharedTaskState>,
    Json(req): Json<CreateTaskRequest>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    info!("📝 Creating task draft: {}", req.title);

    // Create the task
    let task = Task::new(req);
    let task_id = task.id.clone();

    // Generate creation card
    let executor = state.executor();
    let card = executor.create_creation_card(
        &task,
        &task.created_by, // Would be looked up from entity registry
        &task.assigned_to,
    );

    // Store task
    {
        let mut tasks = state.tasks.write().await;
        tasks.insert(task_id.clone(), task.clone());
    }

    // TODO: Publish task.created event to UBL

    info!("✅ Task created: {}", task_id);

    Ok((
        StatusCode::CREATED,
        Json(TaskResponse {
            task,
            card: Some(serde_json::to_value(&card).unwrap_or_default()),
        }),
    ))
}

/// GET /tasks/:id - Get task by ID
async fn get_task(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    let tasks = state.tasks.read().await;
    
    let task = tasks
        .get(&task_id)
        .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?;

    Ok(Json(TaskResponse {
        task: task.clone(),
        card: None, // Card returned only on state changes
    }))
}

/// POST /tasks/:id/approve - Approve a task draft
async fn approve_task(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
    Json(req): Json<ApproveTaskRequest>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    info!("✅ Approving task: {}", task_id);

    let mut tasks = state.tasks.write().await;
    
    let task = tasks
        .get_mut(&task_id)
        .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?;

    if !task.can_approve() {
        return Err(TaskError::InvalidState(format!(
            "Task cannot be approved in state: {:?}",
            task.status
        )));
    }

    // Apply modifications if any
    if let Some(mods) = req.modifications {
        if let Some(title) = mods.title {
            task.title = title;
        }
        if let Some(description) = mods.description {
            task.description = Some(description);
        }
        if let Some(deadline) = mods.deadline {
            task.deadline = Some(deadline);
        }
        if let Some(cost) = mods.estimated_cost {
            task.estimated_cost = Some(cost);
        }
    }

    task.approve(&req.approved_by);

    // TODO: Publish task.approved event to UBL

    info!("✅ Task approved: {}", task_id);

    Ok(Json(TaskActionResponse {
        success: true,
        task: task.clone(),
        card: None,
        event_id: Some(format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())),
    }))
}

/// POST /tasks/:id/reject - Reject a task draft
async fn reject_task(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
    Json(req): Json<RejectTaskRequest>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    info!("❌ Rejecting task: {}", task_id);

    let mut tasks = state.tasks.write().await;
    
    let task = tasks
        .get_mut(&task_id)
        .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?;

    if !task.can_approve() {
        return Err(TaskError::InvalidState(format!(
            "Task cannot be rejected in state: {:?}",
            task.status
        )));
    }

    task.reject(&req.rejected_by, &req.reason);

    // TODO: Publish task.rejected event to UBL

    Ok(Json(TaskActionResponse {
        success: true,
        task: task.clone(),
        card: None,
        event_id: Some(format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())),
    }))
}

/// POST /tasks/:id/execute - Start task execution (synchronous)
async fn execute_task(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    info!("🚀 Executing task: {}", task_id);

    // Get task
    let mut task = {
        let tasks = state.tasks.read().await;
        tasks
            .get(&task_id)
            .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?
            .clone()
    };

    if task.status != TaskStatus::Approved {
        return Err(TaskError::InvalidState(format!(
            "Task must be approved before execution, current state: {:?}",
            task.status
        )));
    }

    // Execute
    let executor = state.executor();
    let result = executor.execute_task(&mut task).await?;

    // Update stored task
    {
        let mut tasks = state.tasks.write().await;
        tasks.insert(task_id.clone(), task.clone());
    }

    // Generate completed card
    let card = executor.create_completed_card(
        &task,
        &result,
        &task.created_by,
        &task.assigned_to,
    );

    Ok(Json(TaskActionResponse {
        success: result.success,
        task,
        card: Some(serde_json::to_value(&card).unwrap_or_default()),
        event_id: Some(format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())),
    }))
}

/// GET /tasks/:id/stream - SSE stream for task progress
async fn execute_task_stream(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
) -> std::result::Result<Sse<impl futures::Stream<Item = std::result::Result<Event, std::convert::Infallible>>>, TaskError> {
    info!("📡 Starting SSE stream for task: {}", task_id);

    // Get task
    let task = {
        let mut tasks = state.tasks.write().await;
        let task = tasks
            .get_mut(&task_id)
            .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?;

        if task.status != TaskStatus::Approved {
            return Err(TaskError::InvalidState(format!(
                "Task must be approved before execution, current state: {:?}",
                task.status
            )));
        }

        task.clone()
    };

    // Create executor and start streaming
    let executor = state.executor();
    let (mut rx, _handle) = executor.execute_with_progress(task).await?;

    // Create SSE stream
    let stream = async_stream::stream! {
        while let Some(msg) = rx.recv().await {
            let event = match msg {
                TaskProgressMessage::Progress(update) => {
                    Event::default()
                        .event("progress")
                        .data(serde_json::to_string(&update).unwrap_or_default())
                }
                TaskProgressMessage::Log(log) => {
                    Event::default()
                        .event("log")
                        .data(serde_json::to_string(&log).unwrap_or_default())
                }
                TaskProgressMessage::Completed(result) => {
                    Event::default()
                        .event("completed")
                        .data(serde_json::to_string(&result).unwrap_or_default())
                }
                TaskProgressMessage::Failed(error) => {
                    Event::default()
                        .event("error")
                        .data(serde_json::json!({
                            "error": error
                        }).to_string())
                }
            };
            yield Ok(event);
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// POST /tasks/:id/accept - Accept completed task
async fn accept_task(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
    Json(req): Json<AcceptTaskRequest>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    info!("✅ Accepting task: {}", task_id);

    let mut tasks = state.tasks.write().await;
    
    let task = tasks
        .get_mut(&task_id)
        .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?;

    if !task.can_accept() {
        return Err(TaskError::InvalidState(format!(
            "Task cannot be accepted in state: {:?}",
            task.status
        )));
    }

    task.accept(&req.accepted_by);

    // TODO: Publish task.accepted event to UBL
    // TODO: Commit to git repository for versioning

    info!("✅ Task accepted and finalized: {}", task_id);

    Ok(Json(TaskActionResponse {
        success: true,
        task: task.clone(),
        card: None,
        event_id: Some(format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())),
    }))
}

/// POST /tasks/:id/dispute - Dispute completed task
async fn dispute_task(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
    Json(req): Json<DisputeTaskRequest>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    info!("⚠️ Disputing task: {}", task_id);

    let mut tasks = state.tasks.write().await;
    
    let task = tasks
        .get_mut(&task_id)
        .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?;

    if !task.can_dispute() {
        return Err(TaskError::InvalidState(format!(
            "Task cannot be disputed in state: {:?}",
            task.status
        )));
    }

    task.dispute(&req.disputed_by, &req.reason);

    // TODO: Publish task.disputed event to UBL

    Ok(Json(TaskActionResponse {
        success: true,
        task: task.clone(),
        card: None,
        event_id: Some(format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())),
    }))
}

/// POST /tasks/:id/cancel - Cancel task
async fn cancel_task(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    info!("🚫 Cancelling task: {}", task_id);

    let mut tasks = state.tasks.write().await;
    
    let task = tasks
        .get_mut(&task_id)
        .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?;

    if task.status == TaskStatus::Accepted || task.status == TaskStatus::Rejected {
        return Err(TaskError::InvalidState(format!(
            "Task cannot be cancelled in terminal state: {:?}",
            task.status
        )));
    }

    task.cancel();

    // TODO: Publish task.cancelled event to UBL

    Ok(Json(TaskActionResponse {
        success: true,
        task: task.clone(),
        card: None,
        event_id: Some(format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())),
    }))
}

/// GET /tasks/:id/card - Get current card for task state
async fn get_task_card(
    State(state): State<SharedTaskState>,
    Path(task_id): Path<String>,
) -> std::result::Result<impl IntoResponse, TaskError> {
    let tasks = state.tasks.read().await;
    
    let task = tasks
        .get(&task_id)
        .ok_or_else(|| TaskError::NotFound(format!("Task not found: {}", task_id)))?;

    let executor = state.executor();

    // Generate appropriate card based on state
    let card: serde_json::Value = match task.status {
        TaskStatus::Draft => {
            let card = executor.create_creation_card(
                task,
                &task.created_by,
                &task.assigned_to,
            );
            serde_json::to_value(&card).unwrap_or_default()
        }
        TaskStatus::Running | TaskStatus::Paused => {
            let card = executor.create_progress_card(
                task,
                &task.created_by,
                &task.assigned_to,
            );
            serde_json::to_value(&card).unwrap_or_default()
        }
        TaskStatus::Completed => {
            // Need result to create completed card - return simplified version
            serde_json::json!({
                "card_type": "task.completed",
                "task_id": task.id,
                "status": "completed",
                "summary": "Task completed, awaiting acceptance"
            })
        }
        _ => {
            serde_json::json!({
                "card_type": "task.status",
                "task_id": task.id,
                "status": format!("{:?}", task.status).to_lowercase()
            })
        }
    };

    Ok(Json(card))
}
