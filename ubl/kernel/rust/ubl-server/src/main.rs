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
mod crypto;
mod webauthn_store;

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

    // Evaluate policy
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
            
            // Process projections if atom data was provided
            if let Some(ref atom_data) = link.atom {
                if let Some(event_type) = atom_data.get("type").and_then(|t| t.as_str()) {
                    let pool = state.pool.clone();
                    let container_id = link.container_id.clone();
                    let atom = atom_data.clone();
                    let entry_hash = entry.entry_hash.clone();
                    let sequence = entry.sequence;
                    
                    // Process projection in background (non-blocking)
                    tokio::spawn(async move {
                        if container_id == "C.Jobs" {
                            let projection = projections::JobsProjection::new(pool);
                            if let Err(e) = projection.process_event(event_type, &atom, &entry_hash, sequence).await {
                                error!("Failed to update jobs projection: {}", e);
                            }
                        } else if container_id == "C.Messenger" {
                            let projection = projections::MessagesProjection::new(pool);
                            if let Err(e) = projection.process_event(event_type, &atom, &entry_hash, sequence).await {
                                error!("Failed to update messages projection: {}", e);
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

/// GET /ledger/:container_id/tail
/// SSE stream with PostgreSQL LISTEN/NOTIFY (PR10)
/// Supports Last-Event-ID header for reconnection (Gemini P2 #7)
async fn route_tail(
    State(state): State<AppState>,
    Path(container_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Parse Last-Event-ID for reconnection support
    let last_event_id = headers
        .get("Last-Event-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());
    
    if let Some(seq) = last_event_id {
        info!("📡 SSE tail requested for: {} (resuming from seq {})", container_id, seq);
    } else {
        info!("📡 SSE tail requested for: {}", container_id);
    }
    
    sse::sse_tail(state.pool.clone(), container_id, last_event_id).await
}

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

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ubl_server=info".parse().unwrap()),
        )
        .init();

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

    let state = AppState {
        ledger: PgLedger::new(pool.clone()),
        pool: pool.clone(),
        policy_registry,
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
        .route("/ledger/:container_id/tail", get(route_tail))
        .route("/atom/:hash", get(route_atom))
        .route("/metrics", get(metrics::metrics_handler))
        .with_state(state.clone())
        .merge(id_routes::id_router().with_state(id_state))
        .merge(id_session_token::router().with_state(state.clone()))
        .merge(repo_routes::router().with_state(state.clone()))
        .nest("/query", projections::projection_router().with_state(projection_state))
        // Console v1.1 (ADR-001) — with step-up WebAuthn
        .merge(console_v1::routes(pool.clone(), webauthn.clone()))
        // Registry v1.1 (ADR-002)
        .merge(registry_v1::routes(pool.clone()))
        // Messenger v1 (C.Messenger boundary)
        .merge(messenger_v1::routes(pool.clone()))
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("🚀 UBL Server v2.1 — ADR-001 + ADR-002 Compliant");
    info!("   Listening: http://{}", addr);
    info!("   Database: {}", database_url.split('@').last().unwrap_or("postgres"));
    info!("   Console v1.1: /v1/policy/permit, /v1/commands/issue, /v1/exec.finish");
    info!("   Registry v1.1: /v1/query/registry/*");
    info!("   Projections: /query/jobs, /query/conversations/:id/messages");
    info!("   Runner pulls from: GET /v1/query/commands?pending=1");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
