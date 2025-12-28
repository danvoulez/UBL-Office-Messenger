//! Registry API v1.1 — ADR-UBL-Registry-002
//!
//! Endpoints:
//! - GET /v1/query/registry/projects     → List projects
//! - GET /v1/query/registry/project/:id  → Project detail

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// State for registry routes
#[derive(Clone)]
pub struct RegistryState {
    pub pool: PgPool,
}

/// Create registry v1.1 routes
pub fn routes(pool: PgPool) -> Router {
    let state = RegistryState { pool };
    Router::new()
        .route("/v1/query/registry/projects", get(list_projects))
        .route("/v1/query/registry/project/:project_id", get(get_project))
        .with_state(state)
}

// ============ Types ============

#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    pub tenant_id: String,
    pub q: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ProjectRow {
    pub tenant_id: String,
    pub project_id: String,
    pub name: String,
    pub owners: serde_json::Value,
    pub visibility: String,
    pub repo_url: String,
    pub last_activity: i64,
}

#[derive(Debug, Deserialize)]
pub struct GetProjectQuery {
    pub tenant_id: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ActivityRow {
    pub tenant_id: String,
    pub project_id: String,
    pub ts: i64,
    pub action: String,
    pub actor: String,
    pub ref_: Option<String>,
    pub commit: Option<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReleaseRow {
    pub tenant_id: String,
    pub project_id: String,
    pub tag: String,
    pub commit: String,
    pub notes_hash: String,
    pub ts: i64,
    pub manifest: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ProjectDetail {
    pub project: ProjectRow,
    pub activity: Vec<ActivityRow>,
    pub releases: Vec<ReleaseRow>,
}

// ============ Handlers ============

/// GET /v1/query/registry/projects — List projects for a tenant
async fn list_projects(
    State(state): State<RegistryState>,
    Query(params): Query<ListProjectsQuery>,
) -> impl IntoResponse {
    let pool = &state.pool;
    
    let projects: Vec<ProjectRow> = if let Some(q) = &params.q {
        let pattern = format!("%{}%", q);
        sqlx::query_as(
            r#"
            SELECT tenant_id, project_id, name, owners, visibility, repo_url, last_activity
            FROM registry_projects
            WHERE tenant_id = $1 AND (name ILIKE $2 OR project_id ILIKE $2)
            ORDER BY last_activity DESC
            LIMIT 100
            "#
        )
        .bind(&params.tenant_id)
        .bind(&pattern)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as(
            r#"
            SELECT tenant_id, project_id, name, owners, visibility, repo_url, last_activity
            FROM registry_projects
            WHERE tenant_id = $1
            ORDER BY last_activity DESC
            LIMIT 100
            "#
        )
        .bind(&params.tenant_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    };
    
    (StatusCode::OK, Json(projects))
}

/// GET /v1/query/registry/project/:id — Get project detail
async fn get_project(
    State(state): State<RegistryState>,
    Path(project_id): Path<String>,
    Query(params): Query<GetProjectQuery>,
) -> impl IntoResponse {
    let pool = &state.pool;
    
    // Get project
    let project: Option<ProjectRow> = sqlx::query_as(
        r#"
        SELECT tenant_id, project_id, name, owners, visibility, repo_url, last_activity
        FROM registry_projects
        WHERE tenant_id = $1 AND project_id = $2
        "#
    )
    .bind(&params.tenant_id)
    .bind(&project_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    
    let project = match project {
        Some(p) => p,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Project not found"})),
            ).into_response();
        }
    };
    
    // Get recent activity
    let activity: Vec<ActivityRow> = sqlx::query_as(
        r#"
        SELECT tenant_id, project_id, ts, action, actor, ref as ref_, commit, details
        FROM registry_activity
        WHERE tenant_id = $1 AND project_id = $2
        ORDER BY ts DESC
        LIMIT 50
        "#
    )
    .bind(&params.tenant_id)
    .bind(&project_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    
    // Get releases
    let releases: Vec<ReleaseRow> = sqlx::query_as(
        r#"
        SELECT tenant_id, project_id, tag, commit, notes_hash, ts, manifest
        FROM registry_releases
        WHERE tenant_id = $1 AND project_id = $2
        ORDER BY ts DESC
        LIMIT 20
        "#
    )
    .bind(&params.tenant_id)
    .bind(&project_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    
    let detail = ProjectDetail {
        project,
        activity,
        releases,
    };
    
    (StatusCode::OK, Json(detail)).into_response()
}

