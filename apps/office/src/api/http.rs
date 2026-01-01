//! HTTP API Routes

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State, Query},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    routing::{get, post, delete},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};
use tracing::{error, info};
use uuid::Uuid;

use crate::entity::{Entity, EntityId, EntityParams, EntityType, Instance, EntityRepository};
use crate::session::{Session, SessionType, SessionMode, SessionConfig, Handover};
use crate::context::{ContextFrameBuilder, Narrator};
use crate::governance::{Constitution, DreamingCycle, DreamingConfig, Simulation, SimulationConfig, Action};
use crate::ubl_client::UblClient;
use crate::llm::{LlmProvider, LlmRequest, LlmMessage, SmartRouter, ProviderProfile, default_profiles};
use crate::job_executor::{JobExecutor, types as job_types};
use crate::routes::{ws, deploy};
use crate::{OfficeConfig, OfficeError};

/// Application state
pub struct AppState {
    pub config: OfficeConfig,
    pub ubl_client: Arc<UblClient>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub smart_router: Arc<SmartRouter>,
    pub entity_repository: Arc<EntityRepository>,
    pub job_executor: Arc<JobExecutor>,
    pub entities: HashMap<EntityId, Entity>,
    pub sessions: HashMap<String, Session>,
    pub instances: HashMap<String, Instance>,
    pub handovers: HashMap<EntityId, Vec<Handover>>,
}

impl AppState {
    pub fn new(
        config: OfficeConfig,
        ubl_client: Arc<UblClient>,
        llm_provider: Arc<dyn LlmProvider>,
    ) -> Self {
        // Create entity repository (Chair keeper)
        let entity_repository = Arc::new(EntityRepository::new(
            ubl_client.clone(),
            &config.ubl.container_id,
        ));

        // Create smart router
        let mut router = SmartRouter::new();
        let profiles = default_profiles();
        
        // Register the main provider
        if let Some(profile) = profiles.get("anthropic") {
            router.register(llm_provider.clone(), profile.clone());
        }
        
        let smart_router = Arc::new(router);

        // Create job executor
        let job_executor = Arc::new(JobExecutor::new(
            ubl_client.clone(),
            entity_repository.clone(),
            smart_router.clone(),
            &config.ubl.container_id,
        ));

        Self {
            config,
            ubl_client,
            llm_provider,
            smart_router,
            entity_repository,
            job_executor,
            entities: HashMap::new(),
            sessions: HashMap::new(),
            instances: HashMap::new(),
            handovers: HashMap::new(),
        }
    }
}

pub type SharedState = Arc<RwLock<AppState>>;

/// Create the router
pub fn create_router(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create workspace and deploy routers with state access
    // Note: We'll access UBL client from state inside the handlers
    let ws_state = state.clone();
    let deploy_state = state.clone();
    
    let ws_router = Router::new()
        .route("/office/ws/test", axum::routing::post(ws_test_handler))
        .route("/office/ws/build", axum::routing::post(ws_build_handler))
        .with_state(ws_state);
    
    let deploy_router = Router::new()
        .route("/office/deploy", axum::routing::post(deploy_handler))
        .with_state(deploy_state);

    Router::new()
        // Health
        .route("/health", get(health))
        
        // Workspace and Deploy routes (Prompt 1: Office front-door)
        .merge(ws_router)
        .merge(deploy_router)

        // Entities
        .route("/entities", post(create_entity))
        .route("/entities", get(list_entities))
        .route("/entities/:id", get(get_entity))
        .route("/entities/:id", delete(delete_entity))

        // Sessions
        .route("/entities/:id/sessions", post(create_session))
        .route("/entities/:id/sessions/:sid", get(get_session))
        .route("/entities/:id/sessions/:sid", delete(end_session))
        .route("/entities/:id/sessions/:sid/message", post(send_message))
        .route("/entities/:id/sessions/:sid/handover", post(create_handover))
        .route("/entities/:id/handovers", get(list_handovers))
        .route("/entities/:id/handovers/latest", get(get_latest_handover))

        // Dreaming
        .route("/entities/:id/dream", post(trigger_dream))
        .route("/entities/:id/memory", get(get_memory))

        // Constitution
        .route("/entities/:id/constitution", post(update_constitution))
        .route("/entities/:id/constitution", get(get_constitution))

        // Simulation
        .route("/simulate", post(simulate_action))

        // Jobs
        .route("/jobs/execute", post(execute_job))
        .route("/jobs/execute/stream", post(execute_job_stream))
        .route("/jobs/:job_id/status", get(get_job_status))
        
        // Approvals
        .route("/approvals", get(list_approvals))
        .route("/approvals/:id", post(submit_approval))

        // Affordances
        .route("/affordances", get(list_affordances))
        .route("/affordances/:id", get(get_affordance))

        // Gateway-facing endpoints
        .route("/v1/office/ingest_message", post(ingest_message))
        .route("/v1/office/job_action", post(handle_job_action))

        .layer(cors)
        .with_state(state)
}

// ============ Health ============

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "office",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// ============ Entities ============

#[derive(Debug, Deserialize)]
struct CreateEntityRequest {
    name: String,
    entity_type: EntityType,
    guardian_id: Option<String>,
    constitution: Option<Constitution>,
    baseline_narrative: Option<String>,
}

async fn create_entity(
    State(state): State<SharedState>,
    Json(req): Json<CreateEntityRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let params = EntityParams {
        name: req.name,
        entity_type: req.entity_type,
        guardian_id: req.guardian_id,
        constitution: req.constitution,
        baseline_narrative: req.baseline_narrative,
        metadata: None,
    };

    let entity = Entity::new(params)?;
    let id = entity.id.clone();

    let mut state = state.write().await;
    state.entities.insert(id.clone(), entity.clone());

    info!("Created entity: {}", id);

    Ok((StatusCode::CREATED, Json(entity)))
}

async fn list_entities(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let state = state.read().await;
    let entities: Vec<Entity> = state.entities.values().cloned().collect();
    Json(entities)
}

async fn get_entity(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state = state.read().await;
    let entity = state.entities.get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Entity not found: {}", id)))?;
    Ok(Json(entity.clone()))
}

async fn delete_entity(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let mut state = state.write().await;

    if let Some(entity) = state.entities.get_mut(&id) {
        entity.archive();
        info!("Archived entity: {}", id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(format!("Entity not found: {}", id)))
    }
}

// ============ Sessions ============

#[derive(Debug, Deserialize)]
struct CreateSessionRequest {
    session_type: SessionType,
    session_mode: Option<SessionMode>,
    token_budget: Option<u64>,
    initiator: String,
}

async fn create_session(
    State(state): State<SharedState>,
    Path(entity_id): Path<String>,
    Json(req): Json<CreateSessionRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let mut state = state.write().await;

    // Check entity exists
    let entity = state.entities.get(&entity_id)
        .ok_or_else(|| ApiError::NotFound(format!("Entity not found: {}", entity_id)))?
        .clone();

    if !entity.is_active() {
        return Err(ApiError::BadRequest("Entity is not active".to_string()));
    }

    // Create session config
    let mode = req.session_mode.unwrap_or(SessionMode::Commitment);
    let config = SessionConfig {
        session_type: req.session_type,
        session_mode: mode,
    };

    let mut session = Session::new(entity_id.clone(), config, req.initiator);

    if let Some(budget) = req.token_budget {
        session = session.with_budget(budget);
    }

    // Fix #9: UBL is the authoritative source for handovers
    // We no longer use local memory - the ContextFrameBuilder will fetch from UBL
    // This ensures the "Dignity Trajectory" is based on the immutable ledger

    // Build context frame (fetches handover from UBL via get_last_handover)
    let frame = ContextFrameBuilder::new(
        entity.clone(),
        req.session_type,
        state.ubl_client.clone(),
    )
    .build()
    .await?;

    // Note: frame.previous_handover is now always from UBL
    // Local handovers are deprecated - they should be committed to UBL at session end

    // Generate narrative
    let narrator = Narrator::default();
    let narrative = narrator.generate(&frame);

    // Create instance
    let mut instance = Instance::new(
        entity_id.clone(),
        req.session_type,
        mode,
        session.token_budget,
    );
    instance.set_context(frame.clone());
    instance.start();

    session.start();
    session.set_instance(instance.id.clone());

    let session_id = session.id.clone();
    let instance_id = instance.id.clone();

    state.sessions.insert(session_id.clone(), session.clone());
    state.instances.insert(instance_id.clone(), instance);

    info!("Created session: {} for entity: {}", session_id, entity_id);

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "session": session,
        "narrative": narrative,
        "frame_hash": frame.frame_hash
    }))))
}

async fn get_session(
    State(state): State<SharedState>,
    Path((entity_id, session_id)): Path<(String, String)>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state = state.read().await;

    let session = state.sessions.get(&session_id)
        .ok_or_else(|| ApiError::NotFound(format!("Session not found: {}", session_id)))?;

    if session.entity_id != entity_id {
        return Err(ApiError::NotFound("Session not found for entity".to_string()));
    }

    Ok(Json(session.clone()))
}

async fn end_session(
    State(state): State<SharedState>,
    Path((entity_id, session_id)): Path<(String, String)>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let mut state = state.write().await;

    let tokens_consumed = {
        let session = state.sessions.get_mut(&session_id)
            .ok_or_else(|| ApiError::NotFound(format!("Session not found: {}", session_id)))?;

        if session.entity_id != entity_id {
            return Err(ApiError::NotFound("Session not found for entity".to_string()));
        }

        session.complete(None);
        session.tokens_consumed
    };

    // Update entity stats
    if let Some(entity) = state.entities.get_mut(&entity_id) {
        entity.record_session(tokens_consumed);
    }

    info!("Ended session: {}", session_id);

    Ok(StatusCode::NO_CONTENT)
}

// ============ Handovers ============

#[derive(Debug, Deserialize)]
struct CreateHandoverRequest {
    content: String,
    summary: Option<String>,
}

#[derive(Debug, Serialize)]
struct HandoverResponse {
    id: String,
    entity_id: String,
    session_id: String,
    content: String,
    summary: Option<String>,
    created_at: String,
}

async fn create_handover(
    State(state): State<SharedState>,
    Path((entity_id, session_id)): Path<(String, String)>,
    Json(req): Json<CreateHandoverRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state_read = state.read().await;

    // Verify session exists and belongs to entity
    let instance_id = {
        let session = state_read.sessions.get(&session_id)
            .ok_or_else(|| ApiError::NotFound(format!("Session not found: {}", session_id)))?;

        if session.entity_id != entity_id {
            return Err(ApiError::NotFound("Session not found for entity".to_string()));
        }

        session.current_instance_id.clone().unwrap_or_default()
    };

    // Create handover
    let mut handover = Handover::new(
        entity_id.clone(),
        session_id.clone(),
        instance_id.clone(),
        req.content.clone(),
    );

    if let Some(ref summary) = req.summary {
        handover.summary = Some(summary.clone());
    }

    let handover_id = handover.id.clone();
    let created_at = handover.created_at.to_rfc3339();

    // Fix #9: Commit handover to UBL ledger instead of local memory
    // Handover is committed as part of session.completed event
    // Store temporarily for inclusion in session completion
    let handover_json = serde_json::json!({
        "handover_id": handover_id.clone(),
        "content": req.content,
        "summary": req.summary,
        "instance_id": instance_id,
        "created_at": created_at,
    });
    
    // Store in local memory temporarily (will be committed when session ends)
    drop(state_read);
    let mut state_write = state.write().await;
    state_write.handovers.entry(entity_id.clone())
        .or_insert_with(Vec::new)
        .push(handover);

    info!("Created handover: {} for session: {} (pending UBL commit on session end)", handover_id, session_id);

    Ok((StatusCode::CREATED, Json(HandoverResponse {
        id: handover_id,
        entity_id,
        session_id,
        content: req.content,
        summary: req.summary,
        created_at,
    })))
}

async fn list_handovers(
    State(state): State<SharedState>,
    Path(entity_id): Path<String>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state = state.read().await;

    // Fix #9: Fetch handovers from UBL ledger (authoritative source)
    let ubl_handovers = state.ubl_client.get_handovers(&entity_id, 50).await
        .unwrap_or_default();
    
    let handovers: Vec<HandoverResponse> = ubl_handovers.iter().map(|ho| HandoverResponse {
        id: ho.id.clone(),
        entity_id: ho.entity_id.clone(),
        session_id: ho.session_id.clone(),
        content: ho.content.clone(),
        summary: ho.summary.clone(),
        created_at: ho.created_at.to_rfc3339(),
    }).collect();

    Ok(Json(handovers))
}

async fn get_latest_handover(
    State(state): State<SharedState>,
    Path(entity_id): Path<String>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state = state.read().await;

    // Fix #9: Fetch latest handover from UBL ledger (authoritative source)
    let handover_content = state.ubl_client.get_last_handover(&entity_id).await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch handover: {}", e)))?;

    match handover_content {
        Some(content) => Ok(Json(serde_json::json!({
            "ok": true,
            "data": {
                "entity_id": entity_id,
                "content": content,
            }
        }))),
        None => Err(ApiError::NotFound("No handovers found".to_string())),
    }
}

#[derive(Debug, Deserialize)]
struct SendMessageRequest {
    content: String,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    response: String,
    tokens_used: u64,
    session_remaining: u64,
}

async fn send_message(
    State(state): State<SharedState>,
    Path((entity_id, session_id)): Path<(String, String)>,
    Json(req): Json<SendMessageRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let (narrative, remaining_budget, current_instance_id, llm_provider) = {
        let state_guard = state.read().await;

        let session = state_guard.sessions.get(&session_id)
            .ok_or_else(|| ApiError::NotFound(format!("Session not found: {}", session_id)))?;

        if session.entity_id != entity_id {
            return Err(ApiError::NotFound("Session not found for entity".to_string()));
        }

        if !session.is_active() {
            return Err(ApiError::BadRequest("Session is not active".to_string()));
        }

        let instance_id = session.current_instance_id.clone().unwrap_or_default();

        // Get context from instance
        let instance = state_guard.instances.get(&instance_id)
            .ok_or_else(|| ApiError::BadRequest("No active instance".to_string()))?;

        let context = instance.context_frame.as_ref()
            .ok_or_else(|| ApiError::BadRequest("No context frame".to_string()))?;

        // Build narrative
        let narrator = Narrator::default();
        let narrative = narrator.generate(context);
        let remaining = session.remaining_budget();
        let llm_provider = state_guard.llm_provider.clone();

        (narrative, remaining, instance_id, llm_provider)
    };

    // Create LLM request with narrative as system instruction
    let llm_request = LlmRequest::new(vec![
        LlmMessage::user(req.content),
    ])
    .with_system(narrative)  // Use dedicated system field for Gemini compatibility
    .with_max_tokens(remaining_budget as u32);

    // Call LLM
    let response = llm_provider.chat(llm_request).await?;
    let tokens_used = response.usage.total_tokens as u64;

    // Update session and instance
    let remaining = {
        let mut state_guard = state.write().await;
        if let Some(session) = state_guard.sessions.get_mut(&session_id) {
            session.consume_tokens(tokens_used);
        }
        if let Some(instance) = state_guard.instances.get_mut(&current_instance_id) {
            instance.consume_tokens(tokens_used);
        }
        state_guard.sessions.get(&session_id)
            .map(|s| s.remaining_budget())
            .unwrap_or(0)
    };

    Ok(Json(MessageResponse {
        response: response.content,
        tokens_used,
        session_remaining: remaining,
    }))
}

// ============ Dreaming ============

async fn trigger_dream(
    State(state): State<SharedState>,
    Path(entity_id): Path<String>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state_guard = state.read().await;

    let entity = state_guard.entities.get(&entity_id)
        .ok_or_else(|| ApiError::NotFound(format!("Entity not found: {}", entity_id)))?
        .clone();

    let ubl_client = state_guard.ubl_client.clone();
    let llm_provider = state_guard.llm_provider.clone();

    drop(state_guard);

    // Create dreaming cycle
    let config = DreamingConfig::default();
    let dreaming = DreamingCycle::new(config, ubl_client)
        .with_llm_provider(llm_provider);

    // Execute with a copy of memory
    let mut memory = crate::context::Memory::new(entity.baseline_narrative.clone());
    let result = dreaming.execute(&entity_id, &mut memory).await?;

    // Update entity
    let mut state_guard = state.write().await;
    if let Some(entity) = state_guard.entities.get_mut(&entity_id) {
        entity.update_baseline(result.new_baseline.clone());
        entity.record_dream();
    }

    info!("Completed dreaming cycle for entity: {}", entity_id);

    Ok(Json(result))
}

async fn get_memory(
    State(state): State<SharedState>,
    Path(entity_id): Path<String>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state = state.read().await;

    let entity = state.entities.get(&entity_id)
        .ok_or_else(|| ApiError::NotFound(format!("Entity not found: {}", entity_id)))?;

    Ok(Json(serde_json::json!({
        "entity_id": entity_id,
        "baseline_narrative": entity.baseline_narrative,
        "total_sessions": entity.total_sessions,
        "total_tokens": entity.total_tokens_consumed,
        "last_dream": entity.last_dream_at
    })))
}

// ============ Constitution ============

async fn update_constitution(
    State(state): State<SharedState>,
    Path(entity_id): Path<String>,
    Json(constitution): Json<Constitution>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let mut state = state.write().await;

    let entity = state.entities.get_mut(&entity_id)
        .ok_or_else(|| ApiError::NotFound(format!("Entity not found: {}", entity_id)))?;

    entity.update_constitution(constitution.clone());

    info!("Updated constitution for entity: {}", entity_id);

    Ok(Json(constitution))
}

async fn get_constitution(
    State(state): State<SharedState>,
    Path(entity_id): Path<String>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state = state.read().await;

    let entity = state.entities.get(&entity_id)
        .ok_or_else(|| ApiError::NotFound(format!("Entity not found: {}", entity_id)))?;

    Ok(Json(entity.constitution.clone()))
}

// ============ Simulation ============

#[derive(Debug, Deserialize)]
struct SimulateRequest {
    action_id: String,
    action_name: String,
    parameters: serde_json::Value,
    risk_score: f32,
    entity_id: String,
}

async fn simulate_action(
    State(state): State<SharedState>,
    Json(req): Json<SimulateRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let simulation = Simulation::new(SimulationConfig::default());

    let action = Action {
        id: req.action_id,
        name: req.action_name,
        parameters: req.parameters,
        risk_score: req.risk_score,
        entity_id: req.entity_id,
    };

    let result = simulation.simulate(action).await?;

    Ok(Json(result))
}

// ============ Affordances ============

async fn list_affordances(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let state = state.read().await;

    if let Some(entity_id) = params.get("entity_id") {
        let affordances = state.ubl_client.get_affordances(entity_id).await
            .unwrap_or_default();
        Ok(Json(affordances))
    } else {
        Ok(Json(Vec::<crate::ubl_client::UblAffordance>::new()))
    }
}

async fn get_affordance(
    State(_state): State<SharedState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    // In real implementation, would fetch specific affordance
    Err(ApiError::NotFound(format!("Affordance not found: {}", id)))
}

// ============ Job Execution ============

#[derive(Debug, Deserialize)]
struct ExecuteJobRequest {
    job: job_types::Job,
    conversation_context: job_types::ConversationContext,
}

async fn execute_job(
    State(state): State<SharedState>,
    Json(req): Json<ExecuteJobRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let job_executor = {
        let state_guard = state.read().await;
        state_guard.job_executor.clone()
    };

    let job_id = req.job.id.clone();
    info!("Executing job: {}", job_id);

    // Execute job (this may take a while)
    match job_executor.execute_job(req.job, req.conversation_context).await {
        Ok(result) => {
            info!("Job {} completed: success={}", job_id, result.success);
            Ok(Json(result))
        }
        Err(e) => {
            info!("Job {} failed: {}", job_id, e);
            Ok(Json(job_types::JobResult {
                job_id,
                success: false,
                summary: None,
                output: None,
                artifacts: vec![],
                tokens_used: 0,
                value_created: None,
                duration_seconds: 0,
                error: Some(e.to_string()),
            }))
        }
    }
}

/// Execute job with SSE streaming
async fn execute_job_stream(
    State(state): State<SharedState>,
    Json(req): Json<ExecuteJobRequest>,
) -> impl IntoResponse {
    use axum::response::sse::{Event, KeepAlive, Sse};
    use futures::stream::{self, Stream};
    use std::convert::Infallible;
    use tokio_stream::StreamExt;

    let job_executor = {
        let state_guard = state.read().await;
        state_guard.job_executor.clone()
    };

    let job_id = req.job.id.clone();
    info!("Executing job (streaming): {}", job_id);

    // Create SSE stream
    let stream = async_stream::stream! {
        // Send start event
        yield Ok::<_, Infallible>(Event::default()
            .event("progress")
            .data(serde_json::json!({
                "job_id": job_id.clone(),
                "status": "starting",
                "progress_percent": 0,
                "message": "Initializing..."
            }).to_string()));

        // Execute job
        match job_executor.execute_job(req.job, req.conversation_context).await {
            Ok(result) => {
                yield Ok(Event::default()
                    .event("complete")
                    .data(serde_json::to_string(&result).unwrap_or_default()));
            }
            Err(e) => {
                yield Ok(Event::default()
                    .event("error")
                    .data(serde_json::json!({
                        "job_id": job_id.clone(),
                        "success": false,
                        "error": e.to_string()
                    }).to_string()));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn get_job_status(
    State(_state): State<SharedState>,
    Path(job_id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    // TODO: Track job status in memory or via UBL
    Err(ApiError::NotFound(format!("Job status not tracked: {}", job_id)))
}

// ============ Approvals ============

/// List pending approvals
async fn list_approvals(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // TODO: Implement approval storage/retrieval from UBL
    // For now, return empty list
    let approvals: Vec<job_types::ApprovalRequest> = vec![];
    Json(approvals)
}

#[derive(Debug, Deserialize)]
struct SubmitApprovalRequest {
    decision: String,  // "approved", "rejected", "request_changes"
    decided_by: String,
    reason: Option<String>,
}

/// Submit an approval decision
async fn submit_approval(
    State(state): State<SharedState>,
    Path(approval_id): Path<String>,
    Json(req): Json<SubmitApprovalRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let decision = job_types::ApprovalDecision {
        approval_id: approval_id.clone(),
        job_id: String::new(), // Would be looked up from approval
        decision: req.decision,
        decided_by: req.decided_by,
        decided_at: Utc::now(),
        reason: req.reason,
    };

    // TODO: Publish decision to UBL and notify waiting job executor
    // For now, just acknowledge
    info!("Approval decision submitted: {} - {}", approval_id, decision.decision);

    Ok((StatusCode::OK, Json(decision)))
}

// ============ Gateway-facing Endpoints ============

#[derive(Debug, Deserialize)]
struct IngestMessageRequest {
    conversation_id: String,
    message_id: String,
    from: String,
    content: String,
    tenant_id: String,
}

#[derive(Debug, Serialize)]
struct IngestMessageResponse {
    action: MessageAction,
    reply_content: Option<String>,
    job_id: Option<String>,
    card: Option<serde_json::Value>,
    event_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum MessageAction {
    Reply,
    ProposeJob,
    None,
}

/// POST /v1/office/ingest_message
/// Receive message from Gateway, decide reply vs job proposal
async fn ingest_message(
    State(state): State<SharedState>,
    Json(req): Json<IngestMessageRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    info!("📨 Office: ingest_message conversation={} message={}", 
          req.conversation_id, req.message_id);

    // 1. Build conversation context from UBL
    // TODO: Query UBL projections for conversation state
    let conversation_context = crate::job_executor::ConversationContextBuilder::new(
        req.conversation_id.clone()
    )
    .with_participants(vec![req.from.clone()])
    .with_recent_messages(vec![crate::job_executor::types::Message {
        id: req.message_id.clone(),
        from: req.from.clone(),
        content: req.content.clone(),
        timestamp: Utc::now(),
    }])
    .build();

    // 2. Use LLM to decide action
    // For now, simple heuristic: if message contains action words, propose job
    let action = if req.content.to_lowercase().contains("create") 
        || req.content.to_lowercase().contains("schedule")
        || req.content.to_lowercase().contains("send")
        || req.content.to_lowercase().contains("organize") {
        MessageAction::ProposeJob
    } else {
        MessageAction::Reply
    };

    match action {
        MessageAction::ProposeJob => {
            // 3. Create job proposal
            let job_id = format!("job_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
            
            // 4. Generate FormalizeCard
            let card = crate::job_executor::cards::FormalizeCard {
                base: crate::job_executor::cards::CardBase {
                    card_id: format!("card_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string()),
                    job_id: job_id.clone(),
                    version: "v1".to_string(),
                    title: format!("Proposed: {}", req.content.chars().take(50).collect::<String>()),
                    summary: Some(req.content.clone()),
                    state: crate::job_executor::fsm::JobState::Proposed,
                    created_at: Utc::now(),
                    conversation_id: req.conversation_id.clone(),
                    tenant_id: req.tenant_id.clone(),
                    owner: crate::job_executor::cards::CardActor {
                        entity_id: "ent_office".to_string(),
                        display_name: "Office".to_string(),
                        actor_type: crate::job_executor::cards::ActorType::Agent,
                    },
                    author: crate::job_executor::cards::CardActor {
                        entity_id: "ent_office".to_string(),
                        display_name: "Office".to_string(),
                        actor_type: crate::job_executor::cards::ActorType::Agent,
                    },
                    buttons: crate::job_executor::cards::FormalizeCard::default_buttons(&job_id),
                },
                job: crate::job_executor::cards::JobDefinition {
                    job_id: job_id.clone(),
                    goal: req.content.clone(),
                    description: Some(req.content.clone()),
                    priority: Some(crate::job_executor::cards::Priority::Normal),
                    due_at: None,
                    inputs_needed: None,
                    expected_outputs: None,
                    constraints: None,
                    sla_hint: None,
                },
                plan_hint: None,
            };

            // 5. Emit job.created event to UBL
            // TODO: Actually commit to UBL via ubl_client
            let event_ids = vec![format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())];

            Ok(Json(IngestMessageResponse {
                action: MessageAction::ProposeJob,
                reply_content: None,
                job_id: Some(job_id),
                card: Some(serde_json::to_value(&card).unwrap()),
                event_ids,
            }))
        }
        MessageAction::Reply => {
            // Generate simple reply
            let reply = format!("I received your message: {}", req.content);
            
            // TODO: Emit message.sent event to UBL
            let event_ids = vec![format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())];

            Ok(Json(IngestMessageResponse {
                action: MessageAction::Reply,
                reply_content: Some(reply),
                job_id: None,
                card: None,
                event_ids,
            }))
        }
        MessageAction::None => {
            Ok(Json(IngestMessageResponse {
                action: MessageAction::None,
                reply_content: None,
                job_id: None,
                card: None,
                event_ids: vec![],
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
struct JobActionRequest {
    job_id: String,
    action_type: String,
    button_id: String,
    card_id: String,
    input_data: Option<serde_json::Value>,
    tenant_id: String,
}

#[derive(Debug, Serialize)]
struct JobActionResponse {
    success: bool,
    updated_card: Option<serde_json::Value>,
    event_ids: Vec<String>,
}

/// POST /v1/office/job_action
/// Handle job button actions (approve/reject/provide_input)
async fn handle_job_action(
    State(state): State<SharedState>,
    Json(req): Json<JobActionRequest>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    info!("🔧 Office: job_action job={} action={}", req.job_id, req.action_type);

    // 1. Validate card provenance (button exists in prior card)
    // TODO: Query UBL for prior message.sent event with card.card_id
    // For now, assume valid

    // 2. Update job state via FSM
    let mut fsm = crate::job_executor::fsm::JobStateTracker::with_state(
        crate::job_executor::fsm::JobState::Proposed
    );

    let transition_result = match req.action_type.as_str() {
        "approve" => fsm.approve(),
        "reject" => fsm.reject(),
        "provide_input" => fsm.resume(),
        _ => Err(crate::OfficeError::JobTransitionError("Unknown action".to_string())),
    };

    match transition_result {
        Ok(_) => {
            // 3. Emit events to UBL
            // TODO: Actually commit to UBL
            let event_ids = vec![format!("evt_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_string())];

            Ok(Json(JobActionResponse {
                success: true,
                updated_card: None, // TODO: Generate updated card
                event_ids,
            }))
        }
        Err(e) => {
            error!("Job action failed: {}", e);
            Err(ApiError::BadRequest(e.to_string()))
        }
    }
}

// ============ Error Handling ============

#[derive(Debug)]
enum ApiError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

impl From<OfficeError> for ApiError {
    fn from(err: OfficeError) -> Self {
        match err {
            OfficeError::EntityNotFound(msg) => ApiError::NotFound(msg),
            OfficeError::SessionError(msg) => ApiError::BadRequest(msg),
            _ => ApiError::Internal(err.to_string()),
        }
    }
}

// Workspace and Deploy route handlers (Prompt 1: Office front-door)
async fn ws_test_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(body): Json<crate::types::WsTestBody>,
) -> Result<impl IntoResponse, ApiError> {
    let state_read = state.read().await;
    let ubl_client = state_read.ubl_client.clone();
    let ubl_base = state_read.config.ubl.endpoint.clone();
    drop(state_read);
    
    let office_state = ws::OfficeState {
        ubl_base,
        ubl_client,
    };
    
    ws::ws_test(State(office_state), headers, Json(body)).await
        .map_err(|(status, msg)| ApiError::Internal(format!("{}: {}", status, msg)))
}

async fn ws_build_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(body): Json<crate::types::WsBuildBody>,
) -> Result<impl IntoResponse, ApiError> {
    let state_read = state.read().await;
    let ubl_client = state_read.ubl_client.clone();
    let ubl_base = state_read.config.ubl.endpoint.clone();
    drop(state_read);
    
    let office_state = ws::OfficeState {
        ubl_base,
        ubl_client,
    };
    
    ws::ws_build(State(office_state), headers, Json(body)).await
        .map_err(|(status, msg)| ApiError::Internal(format!("{}: {}", status, msg)))
}

async fn deploy_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(body): Json<crate::types::DeployBody>,
) -> Result<impl IntoResponse, ApiError> {
    let state_read = state.read().await;
    let ubl_client = state_read.ubl_client.clone();
    let ubl_base = state_read.config.ubl.endpoint.clone();
    drop(state_read);
    
    let office_state = deploy::OfficeState {
        ubl_base,
        ubl_client,
    };
    
    deploy::deploy(State(office_state), headers, Json(body)).await
        .map_err(|(status, msg)| ApiError::Internal(format!("{}: {}", status, msg)))
}
