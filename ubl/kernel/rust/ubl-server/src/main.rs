//! # UBL Server v2.1 + Console v1.1 + Registry v1.1
//!
//! HTTP API com PostgreSQL append-only ledger
//! SPEC-UBL-LEDGER v1.0 compliant
//! ADR-UBL-Console-001 v1.1 + ADR-UBL-Registry-002 v1.1
//!
//! Core Routes:
//! - GET  /health
//! - GET  /state/:container_id  
//! - POST /link/validate
//! - POST /link/commit
//! - GET  /ledger/:container_id/tail (SSE)
//! - GET  /atom/:hash
//!
//! Console v1.1 (ADR-001):
//! - POST /v1/policy/permit       → Issue Permit
//! - POST /v1/commands/issue      → Register Command
//! - GET  /v1/query/commands      → List pending (Runner pulls)
//! - POST /v1/exec.finish         → Register Receipt
//!
//! Registry v1.1 (ADR-002):
//! - GET  /v1/query/registry/projects
//! - GET  /v1/query/registry/project/:id
//!
//! Identity:
//! - POST /id/agents (create LLM/App)
//! - POST /id/agents/{sid}/asc (issue ASC)
//! - GET  /id/whoami

mod db;
mod sse;
mod id_db;
mod id_routes;
mod auth;
mod rate_limit;
mod metrics;
mod otel_tracing;
mod id_ledger;
mod id_session_token;
mod repo_routes;
mod middleware_require_stepup;
mod projections;
mod pact_db;
mod policy_registry;
mod console_v1;
mod registry_v1;
mod messenger_v1;
mod messenger_gateway;
mod policy;
mod crypto;
mod webauthn_store;
mod keystore;
mod snapshots;
mod tenant;

use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use db::{LedgerEntry, LinkDraft, PgLedger, TangencyError, PactProofDraft, PactSignatureDraft};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};
use webauthn_rs::prelude::*;

// UBL Kernel for cryptographic verification
use ubl_kernel::verify as verify_signature;

// ============================================================================
// APPLICATION STATE
// ============================================================================

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    ledger: PgLedger,
    policy_registry: std::sync::Arc<policy_registry::PolicyRegistry>,
    tail_tx: tokio::sync::broadcast::Sender<(String, String)>, // (container_id, sequence_str) - matches TailBus
    tail_bus: sse::TailBus, // New: simplified SSE bus
}

// ============================================================================
// TYPES
// ============================================================================

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct Decision {
    decision: &'static str,
}

#[derive(Serialize)]
struct CommitSuccess {
    ok: bool,
    entry: LedgerEntry,
}

#[derive(Serialize)]
struct StateResponse {
    container_id: String,
    sequence: i64,
    last_hash: String,
    entry_count: i64,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// GET /health
async fn route_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: "2.0.0+postgres",
    })
}

/// GET /state/:container_id
async fn route_state(
    State(state): State<AppState>,
    Path(container_id): Path<String>,
) -> Result<Json<StateResponse>, (StatusCode, String)> {
    match state.ledger.get_state(&container_id).await {
        Ok(entry) => {
            // Get entry count
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM ledger_entry WHERE container_id = $1"
            )
            .bind(&container_id)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(0);

            Ok(Json(StateResponse {
                container_id: entry.container_id,
                sequence: entry.sequence,
                last_hash: entry.entry_hash,
                entry_count: count,
            }))
        }
        Err(_) => {
            // Genesis state
            Ok(Json(StateResponse {
                container_id,
                sequence: 0,
                last_hash: "0x00".to_string(),
                entry_count: 0,
            }))
        }
    }
}

/// POST /link/validate
/// Basic validation - in production, inject full Membrane here
async fn route_validate(
    State(_state): State<AppState>,
    Json(_link): Json<LinkDraft>,
) -> Json<Decision> {
    // TODO: Apply SPEC-UBL-MEMBRANE v1.0 §V1-V9 validations
    // For now, simplified validation
    Json(Decision {
        decision: "Accept",
    })
}

/// POST /link/commit
/// Atomic append with SERIALIZABLE transaction + ASC validation
async fn route_commit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(link): Json<LinkDraft>,
) -> Result<Json<CommitSuccess>, (StatusCode, String)> {
    info!(
        "📝 COMMIT seq={} container={} class={}",
        link.expected_sequence, link.container_id, link.intent_class
    );

    // ASC Validation (PR29)
    if let Some(auth_header) = headers.get("authorization") {
        let auth_str = auth_header.to_str().map_err(|_| {
            (StatusCode::BAD_REQUEST, "Invalid authorization header".to_string())
        })?;

        // Extract SID
        let sid = auth::extract_sid_from_header(auth_str).map_err(|e| {
            error!("❌ AUTH ERROR: {}", e.message());
            (e.status_code(), e.message())
        })?;

        // Validate ASC
        let asc_context = auth::validate_asc(&state.pool, &sid).await.map_err(|e| {
            error!("❌ ASC VALIDATION FAILED: {}", e.message());
            (e.status_code(), e.message())
        })?;

        // Validate commit scopes
        auth::validate_commit_scopes(
            &asc_context,
            &link.container_id,
            &link.intent_class,
            &link.physics_delta,
        ).map_err(|e| {
            error!("❌ SCOPE VIOLATION: {}", e.message());
            (e.status_code(), e.message())
        })?;

        info!("✅ ASC VALIDATED sid={} containers={:?}", sid, asc_context.containers);
    } else {
        // No ASC provided - allow for now (TODO: make required in production)
        info!("⚠️  No ASC provided (dev mode - allowing)");
    }

    // ========================================================================
    // SIGNATURE VERIFICATION (SPEC-UBL-MEMBRANE v1.0 §V2)
    // ========================================================================
    // Build the canonical signing bytes (link without signature)
    let signing_data = serde_json::json!({
        "version": link.version,
        "container_id": link.container_id,
        "expected_sequence": link.expected_sequence,
        "previous_hash": link.previous_hash,
        "atom_hash": link.atom_hash,
        "intent_class": link.intent_class,
        "physics_delta": link.physics_delta,
        "pact": link.pact,
    });
    
    // Canonicalize for verification (sorted keys, no whitespace)
    let signing_bytes = match ubl_atom::canonicalize(&signing_data) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("❌ CANONICALIZATION FAILED: {}", e);
            return Err((StatusCode::BAD_REQUEST, format!("CanonicalizeError: {}", e)));
        }
    };
    
    // Verify Ed25519 signature
    if let Err(e) = verify_signature(&link.author_pubkey, &signing_bytes, &link.signature) {
        error!("❌ SIGNATURE INVALID: author={} error={}", &link.author_pubkey[..16], e);
        return Err((StatusCode::FORBIDDEN, "SignatureInvalid".to_string()));
    }
    
    info!("✅ SIGNATURE VERIFIED: author={}", &link.author_pubkey[..16]);

    // POLICY EVALUATION (SPEC-UBL-POLICY v1.0)
    // Evaluate policy BEFORE pact validation
    let physics_delta: i128 = link.physics_delta.parse().unwrap_or(0);
    let current_time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    // Get actor from ASC or default
    let actor = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| auth::extract_sid_from_header(s).ok())
        .unwrap_or_else(|| "anonymous".to_string());

    // Apply Policy Pack v1 checks
    if let Some(ref atom) = link.atom {
        let policy_engine = policy::PolicyEngine::new(state.pool.clone());
        
        // Check for raw PII
        if let Err(e) = policy_engine.check_no_raw_pii(atom) {
            error!("❌ Policy violation: {}", e);
            return Err((StatusCode::FORBIDDEN, format!("PolicyViolation: {}", e)));
        }

        // Check job FSM if this is a job state change
        if let Some(event_type) = atom.get("type").and_then(|t| t.as_str()) {
            if event_type == "job.state_changed" {
                if let (Some(from), Some(to), Some(job_id)) = (
                    atom.get("from_state").and_then(|v| v.as_str()),
                    atom.get("to_state").and_then(|v| v.as_str()),
                    atom.get("job_id").and_then(|v| v.as_str()),
                ) {
                    if let Err(e) = policy_engine.validate_job_fsm(job_id, from, to).await {
                        error!("❌ Policy violation: {}", e);
                        return Err((StatusCode::FORBIDDEN, format!("PolicyViolation: {}", e)));
                    }
                }
            }

            // Check tool pairing
            if event_type == "tool.result" {
                if let Some(tool_call_id) = atom.get("payload")
                    .and_then(|p| p.get("tool_call_id"))
                    .and_then(|v| v.as_str())
                {
                    if let Err(e) = policy_engine.validate_tool_pairing(tool_call_id, event_type).await {
                        error!("❌ Policy violation: {}", e);
                        return Err((StatusCode::FORBIDDEN, format!("PolicyViolation: {}", e)));
                    }
                }
            }
        }
    }

    // Evaluate policy via registry
    let policy_decision = state.policy_registry.evaluate(
        &link.container_id,
        &actor,
        link.atom.as_ref().unwrap_or(&serde_json::json!({})),
        None,
        current_time_ms,
    ).await;

    match &policy_decision {
        Ok(ubl_policy_vm::TranslationDecision::Deny { reason }) => {
            error!("❌ POLICY DENIED: {}", reason);
            return Err((StatusCode::FORBIDDEN, format!("PolicyDenied: {}", reason)));
        }
        Ok(ubl_policy_vm::TranslationDecision::Allow { intent_class, required_pact, .. }) => {
            info!("✅ POLICY ALLOWED: intent_class={} pact={:?}", intent_class, required_pact);
            
            // Check if policy requires a pact that wasn't provided
            if required_pact.is_some() && link.pact.is_none() {
                error!("❌ POLICY REQUIRES PACT: {:?}", required_pact);
                return Err((StatusCode::FORBIDDEN, format!("PolicyRequiresPact: {:?}", required_pact)));
            }
        }
        Err(e) => {
            // Policy evaluation failed - log but continue for now
            warn!("⚠️  Policy evaluation failed: {}. Allowing for compatibility.", e);
        }
    }

    // Pact validation (SPEC-UBL-PACT v1.0)
    if pact_db::requires_pact(&link.intent_class, physics_delta) {
        match &link.pact {
            Some(pact_proof) => {
                // Convert to pact_db types
                let proof = pact_db::PactProofInput {
                    pact_id: pact_proof.pact_id.clone(),
                    signatures: pact_proof.signatures.iter().map(|s| {
                        pact_db::PactSignatureInput {
                            signer: s.signer.clone(),
                            signature: s.signature.clone(),
                        }
                    }).collect(),
                };

                if let Err(e) = pact_db::validate_pact_proof(
                    &state.pool,
                    &proof,
                    &link.container_id,
                    &link.intent_class,
                    &link.atom_hash,
                    physics_delta,
                    current_time_ms,
                ).await {
                    error!("❌ PACT VALIDATION FAILED: {}", e);
                    return Err((StatusCode::FORBIDDEN, format!("PactViolation: {}", e)));
                }
            }
            None => {
                error!("❌ PACT REQUIRED but not provided for {} with delta={}", link.intent_class, physics_delta);
                return Err((StatusCode::FORBIDDEN, "PactRequired".to_string()));
            }
        }
    }

    match state.ledger.append(&link).await {
        Ok(entry) => {
            info!("✅ ACCEPTED seq={} hash={}", entry.sequence, &entry.entry_hash[..8]);
            
            // Broadcast SSE event via TailBus (Postgres NOTIFY will also trigger via trigger)
            state.tail_bus.notify(link.container_id.clone(), entry.sequence);
            
            // Process projections if atom data was provided
            if let Some(atom_data) = link.atom.clone() {
                if let Some(event_type) = atom_data.get("type").and_then(|t| t.as_str()).map(|s| s.to_string()) {
                    let pool = state.pool.clone();
                    let container_id = link.container_id.clone();
                    let atom = atom_data.clone();
                    let entry_hash = entry.entry_hash.clone();
                    let sequence = entry.sequence;
                    
                    // Process projection in background (non-blocking)
                    tokio::spawn(async move {
                        let tenant_id = "default"; // TODO: Extract from atom or session
                        let event_type = event_type.as_str();
                        
                        if container_id == "C.Jobs" {
                            // Update main jobs projection
                            let projection = projections::JobsProjection::new(pool.clone());
                            if let Err(e) = projection.process_event(event_type, &atom, &entry_hash, sequence).await {
                                error!("Failed to update jobs projection: {}", e);
                            }
                            
                            // Update new projection tables
                            let job_events = projections::JobEventsProjection::new(pool.clone());
                            if let Err(e) = job_events.process_event(event_type, &atom, &entry_hash, sequence, tenant_id).await {
                                error!("Failed to update job events projection: {}", e);
                            }
                            
                            // Update artifacts if tool.result
                            if event_type == "tool.result" {
                                let artifacts = projections::ArtifactsProjection::new(pool.clone());
                                if let Err(e) = artifacts.process_event(&atom, &entry_hash, tenant_id).await {
                                    error!("Failed to update artifacts projection: {}", e);
                                }
                            }
                            
                            // Update presence based on job state changes
                            if event_type == "job.state_changed" || event_type == "job.started" || event_type == "job.completed" {
                                let presence = projections::PresenceProjection::new(pool.clone());
                                let job_id = atom.get("job_id").or_else(|| atom.get("id")).and_then(|v| v.as_str());
                                let owner = atom.get("owner_entity_id").or_else(|| atom.get("assigned_to")).and_then(|v| v.as_str());
                                let state = atom.get("to_state").or_else(|| atom.get("state")).and_then(|v| v.as_str());
                                let waiting_on = atom.get("waiting_on").and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());
                                
                                if let Some(entity_id) = owner {
                                    if let Err(e) = presence.recompute_from_job(tenant_id, entity_id, job_id, state, waiting_on).await {
                                        error!("Failed to update presence: {}", e);
                                    }
                                }
                            }
                            
                            // Update activity for any event
                            if let Some(actor) = atom.get("actor").and_then(|a| a.get("entity_id")).and_then(|v| v.as_str())
                                .or_else(|| atom.get("from").and_then(|v| v.as_str()))
                                .or_else(|| atom.get("created_by").and_then(|v| v.as_str()))
                            {
                                let presence = projections::PresenceProjection::new(pool.clone());
                                let _ = presence.update_activity(tenant_id, actor, &entry_hash).await;
                            }
                        } else if container_id == "C.Messenger" {
                            let projection = projections::MessagesProjection::new(pool.clone());
                            if let Err(e) = projection.process_event(event_type, &atom, &entry_hash, sequence).await {
                                error!("Failed to update messages projection: {}", e);
                            }
                            
                            // Update timeline
                            let timeline = projections::TimelineProjection::new(pool.clone());
                            let conversation_id = atom.get("conversation_id").and_then(|v| v.as_str()).unwrap_or_default();
                            if !conversation_id.is_empty() {
                                let item_type = if event_type == "message.created" { "message" } else { "system" };
                                let item_data = atom.clone();
                                if let Err(e) = timeline.add_item(tenant_id, conversation_id, item_type, &item_data, sequence).await {
                                    error!("Failed to update timeline: {}", e);
                                }
                            }
                            
                            // Update activity for message sender
                            if let Some(from) = atom.get("from").and_then(|v| v.as_str()) {
                                let presence = projections::PresenceProjection::new(pool.clone());
                                let _ = presence.update_activity(tenant_id, from, &entry_hash).await;
                            }
                        } else if container_id == "C.Office" {
                            let projection = projections::OfficeProjection::new(pool);
                            if let Err(e) = projection.process_event(event_type, &atom, &entry_hash, sequence).await {
                                error!("Failed to update office projection: {}", e);
                            }
                        }
                    });
                }
            }
            
            Ok(Json(CommitSuccess {
                ok: true,
                entry,
            }))
        }
        Err(TangencyError::RealityDrift) => {
            error!("❌ REJECTED: RealityDrift");
            Err((StatusCode::CONFLICT, "RealityDrift".into()))
        }
        Err(TangencyError::SequenceMismatch) => {
            error!("❌ REJECTED: SequenceMismatch");
            Err((StatusCode::CONFLICT, "SequenceMismatch".into()))
        }
        Err(TangencyError::InvalidVersion) => {
            error!("❌ REJECTED: InvalidVersion");
            Err((StatusCode::BAD_REQUEST, "InvalidVersion".into()))
        }
        Err(TangencyError::InvalidTarget) => {
            error!("❌ REJECTED: InvalidTarget");
            Err((StatusCode::BAD_REQUEST, "InvalidTarget".into()))
        }
        Err(TangencyError::PactViolation(reason)) => {
            error!("❌ REJECTED: PactViolation - {}", reason);
            Err((StatusCode::FORBIDDEN, format!("PactViolation: {}", reason)))
        }
    }
}

// SSE route is now handled by sse::router (simplified version)

/// GET /atom/:hash
/// Fetch atom data by hash (PHASE 3B)
async fn route_atom(
    State(state): State<AppState>,
    Path(atom_hash): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    #[derive(sqlx::FromRow)]
    struct AtomRow {
        atom_data: serde_json::Value,
        container_id: String,
        ts_unix_ms: i64,
    }
    
    let result: Option<AtomRow> = sqlx::query_as(
        "SELECT atom_data, container_id, ts_unix_ms FROM ledger_atom WHERE atom_hash = $1"
    )
    .bind(&atom_hash)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match result {
        Some(row) => {
            let response = serde_json::json!({
                "atom_hash": atom_hash,
                "container_id": row.container_id,
                "atom_data": row.atom_data,
                "ts_unix_ms": row.ts_unix_ms
            });
            Ok(Json(response))
        }
        None => {
            Err((StatusCode::NOT_FOUND, format!("Atom not found: {}", atom_hash)))
        }
    }
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenvy::dotenv().ok();

    // Initialize OpenTelemetry tracing
    let otlp_endpoint = std::env::var("OTLP_ENDPOINT")
        .ok()
        .or_else(|| std::env::var("JAEGER_ENDPOINT").ok());
    
    if let Some(endpoint) = otlp_endpoint.as_deref() {
        if let Err(e) = otel_tracing::init_tracing("ubl-server", "2.0.0", Some(endpoint)) {
            warn!("Failed to initialize OpenTelemetry tracing: {}. Falling back to basic tracing.", e);
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive("ubl_server=info".parse().unwrap()),
                )
                .init();
        } else {
            info!("🔍 OpenTelemetry tracing initialized: {}", endpoint);
        }
    } else {
        // Fallback to basic tracing if OTLP endpoint not configured
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("ubl_server=info".parse().unwrap()),
            )
            .init();
        info!("📝 Basic tracing initialized (OpenTelemetry disabled - set OTLP_ENDPOINT to enable)");
    }

    // Initialize KeyStore (Gemini P0 #1)
    keystore::init();
    info!("🔑 KeyStore initialized");
    
    // Load or create admin key (used for signing permits)
    let _admin_pubkey = keystore::get_public_key_hex("admin");
    info!("🔐 Admin public key: {}", _admin_pubkey);
    
    // Initialize Snapshots (Gemini P1 #4)
    snapshots::init();
    info!("📸 Snapshots initialized");

    // Connect to PostgreSQL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ubl_dev@localhost:5432/ubl_dev".to_string());

    info!("🔌 Connecting to PostgreSQL...");
    let pool = PgPool::connect(&database_url).await?;
    info!("✅ PostgreSQL connected");

    // Initialize policy registry
    let policy_registry = std::sync::Arc::new(policy_registry::PolicyRegistry::with_pool(pool.clone()));
    policy_registry.init_defaults().await;
    
    // Try to load policies from database
    if let Err(e) = policy_registry.load_from_database().await {
        warn!("⚠️  Failed to load policies from database: {}. Using defaults.", e);
    }
    info!("📋 Policy engine initialized");

    // Create TailBus for SSE (simplified - only cid:seq)
    let tail_bus = sse::TailBus::new();
    
    // Postgres LISTEN/NOTIFY integration
    // Note: sqlx doesn't have built-in async LISTEN/NOTIFY
    // The trigger will send NOTIFY, but we'll also notify via TailBus directly in route_commit
    // For full async LISTEN support, consider using tokio-postgres or pg_listen crate
    info!("📡 PostgreSQL NOTIFY trigger 'ubl_tail' will be used (trigger created via migration)");

    // Keep tail_tx for AppState compatibility, but also use TailBus
    let state = AppState {
        ledger: PgLedger::new(pool.clone()),
        pool: pool.clone(),
        policy_registry,
        tail_tx: tail_bus.clone().tx.clone(),
        tail_bus: tail_bus.clone(),
    };

    // Initialize WebAuthn
    let rp_id = std::env::var("WEBAUTHN_RP_ID")
        .unwrap_or_else(|_| "localhost".to_string());
    let rp_origin = std::env::var("WEBAUTHN_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    info!("🔐 WebAuthn: rpId={}, origin={}", rp_id, rp_origin);
    
    let rp_origin_url = Url::parse(&rp_origin)
        .expect("Invalid WEBAUTHN_ORIGIN URL");
    
    let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin_url)
        .expect("Failed to create WebAuthn builder")
        .rp_name("UBL Identity")
        .build()
        .expect("Failed to build WebAuthn");

    // Clone webauthn before it gets moved into id_state
    let webauthn_for_console = webauthn.clone();

    let id_state = id_routes::IdState { 
        pool: pool.clone(),
        webauthn,
        rate_limiter: rate_limit::RateLimiter::new(),
    };

    // Projection state
    let projection_state = projections::ProjectionState {
        pool: pool.clone(),
    };

    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(route_health))
        .route("/state/:container_id", get(route_state))
        .route("/link/validate", post(route_validate))
        .route("/link/commit", post(route_commit))
        .route("/atom/:hash", get(route_atom))
        .with_state(state.clone())
        .merge(metrics::metrics_router())
        .merge(sse::sse_router(tail_bus.clone())) // SSE simplified (only cid:seq)
        .merge(id_routes::id_router().with_state(id_state))
        .merge(id_session_token::router().with_state(state.clone()))
        .merge(repo_routes::router().with_state(state.clone()))
        .nest("/query", projections::projection_router().with_state(projection_state))
        // Console v1.1 (ADR-001) — with step-up WebAuthn
        .merge(console_v1::routes(pool.clone(), webauthn_for_console))
        // Registry v1.1 (ADR-002)
        .merge(registry_v1::routes(pool.clone()))
        // Messenger v1 (C.Messenger boundary)
        .merge(messenger_v1::routes(pool.clone()))
        // Messenger Gateway v1
        .merge(messenger_gateway::routes(
            pool.clone(),
            std::env::var("OFFICE_URL").unwrap_or_else(|_| "http://localhost:8081".to_string())
        ))
        // Tenant Management (C.Tenant)
        .merge(tenant::tenant_routes().with_state(pool.clone()))
        .layer(cors);

    // Prompt 3: Unix Socket support - REQUIRED for security
    if let Ok(unix_path) = std::env::var("UBL_LISTEN_UNIX") {
        use std::path::Path;
        use std::fs;
        use hyper::server::conn::http1::Builder as Http1Builder;
        use hyper_util::rt::TokioIo;
        use tokio::net::UnixListener;
        use tower_service::Service;
        
        let p = Path::new(&unix_path);
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir)?;
        }
        let _ = fs::remove_file(&p); // evita "address in use"

        info!("🚀 UBL Server v2.1 — ADR-001 + ADR-002 Compliant");
        info!("   Listening: unix://{}", unix_path);
        info!("   Database: {}", database_url.split('@').last().unwrap_or("postgres"));
        info!("   Console v1.1: /v1/policy/permit, /v1/commands/issue, /v1/exec.finish");
        info!("   Registry v1.1: /v1/query/registry/*");
        info!("   Projections: /query/jobs, /query/conversations/:id/messages, /query/office/*");
        info!("   Runner pulls from: GET /v1/query/commands?pending=1");

        let listener = UnixListener::bind(p)?;
        let mut make_service = app.into_make_service();
        
        loop {
            let (stream, _) = listener.accept().await?;
            let tower_service = make_service.call(()).await.expect("make_service");
            
            tokio::spawn(async move {
                let io = TokioIo::new(stream);
                
                // Wrap tower service for hyper 1.x compatibility
                let hyper_service = hyper_util::service::TowerToHyperService::new(tower_service);
                
                let http1 = Http1Builder::new();
                
                if let Err(e) = http1.serve_connection(io, hyper_service).await {
                    error!("Error serving Unix Socket connection: {}", e);
                }
            });
        }
    } else {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let addr = format!("0.0.0.0:{}", port);

        info!("🚀 UBL Server v2.1 — ADR-001 + ADR-002 Compliant");
        info!("   Listening: http://{}", addr);
        info!("   Database: {}", database_url.split('@').last().unwrap_or("postgres"));
        info!("   Console v1.1: /v1/policy/permit, /v1/commands/issue, /v1/exec.finish");
        info!("   Registry v1.1: /v1/query/registry/*");
        info!("   Projections: /query/jobs, /query/conversations/:id/messages, /query/office/*");
        info!("   Runner pulls from: GET /v1/query/commands?pending=1");

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
    }
    
    Ok(())
}
