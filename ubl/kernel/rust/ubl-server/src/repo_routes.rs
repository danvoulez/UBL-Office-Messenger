use axum::{Json, extract::State, routing::post, Router};
use serde::{Deserialize, Serialize};
use std::process::Command;
use crate::AppState;
use axum::http::StatusCode;

#[derive(Debug, Deserialize)]
pub struct PresignBody {
    pub tenant: String,
    pub repo: String,
    pub objects: Vec<PresignObject>, // sha256 + size expected
    #[serde(default = "default_ttl")]
    pub ttl_secs: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PresignObject {
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct PresignResult {
    pub object: PresignObject,
    pub put_url: String,
    pub path: String,
}

fn default_ttl() -> u64 { 600 } // 10 minutos

#[derive(Debug, Deserialize)]
pub struct CommitRefBody {
    pub tenant: String,
    pub repo: String,
    pub r#ref: String,
    pub old: String,
    pub new: String,
    pub mode: String, // "ff" | "force"
}

#[derive(Debug, Serialize)]
pub struct CommitRefResult {
    pub status: String,
    pub link_hash: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/repo/presign", post(route_repo_presign))
        .route("/repo/commit-ref", post(route_repo_commit_ref))
}

/// Presign via MinIO 'mc' tool (fallback). You must have an alias configured (e.g., 'ubl').
/// Path: vault-repos/{tenant}/{repo}/objects/{prefix2}/{sha256}
async fn route_repo_presign(
    State(_state): State<AppState>,
    Json(body): Json<PresignBody>,
) -> Result<Json<Vec<PresignResult>>, (StatusCode, String)> {
    let alias = std::env::var("MINIO_ALIAS").unwrap_or_else(|_| "ubl".into());
    let bucket = std::env::var("MINIO_BUCKET_REPOS").unwrap_or_else(|_| "vault-repos".into());
    let mut out = Vec::with_capacity(body.objects.len());
    for o in body.objects.iter() {
        let prefix2 = &o.sha256[..2]; // fanout
        let path = format!("{}/{}/{}/objects/{}/{}", bucket, body.tenant, body.repo, prefix2, o.sha256);
        // mc command: mc share upload --expire=600s ubl/vault-repos/... → returns URL
        let expire = format!("{}s", body.ttl_secs);
        let target = format!("{}/{}", alias, path);
        let cmd = Command::new("mc")
            .args(["share", "upload", "--expire", &expire, &target])
            .output()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("mc not available: {e}")))?;
        if !cmd.status.success() {
            return Err((StatusCode::BAD_GATEWAY, String::from_utf8_lossy(&cmd.stderr).to_string()));
        }
        let stdout = String::from_utf8_lossy(&cmd.stdout).to_string();
        // Heuristic: find first http(s) URL on stdout
        let put_url = stdout
            .split_whitespace()
            .find(|s| s.starts_with("http://") || s.starts_with("https://"))
            .ok_or((StatusCode::BAD_GATEWAY, "mc share output missing URL".to_string()))?
            .to_string();
        out.push(PresignResult { 
            object: PresignObject { sha256: o.sha256.clone(), size: o.size },
            put_url, 
            path: format!("s3://{}", path) 
        });
    }
    Ok(Json(out))
}

/// Commit a ref change (static container: Δ=0). Builds a 'git/ref' atom and forwards to link/commit flow.
async fn route_repo_commit_ref(
    State(_state): State<AppState>,
    Json(body): Json<CommitRefBody>,
) -> Result<Json<CommitRefResult>, (StatusCode, String)> {
    if body.mode != "ff" && body.mode != "force" {
        return Err((StatusCode::BAD_REQUEST, "mode must be 'ff' or 'force'".into()));
    }
    
    // TODO: Build proper LinkDraft with signatures and append to ledger
    // For now, just return success with placeholder hash
    
    let container_id = format!("repo://{}/{}", body.tenant, body.repo);
    tracing::info!(
        container_id = %container_id,
        ref_name = %body.r#ref,
        old = %body.old,
        new = %body.new,
        mode = %body.mode,
        "Repo ref commit (ledger append not yet implemented)"
    );
    
    let link_hash = format!("0x{}", hex::encode(blake3::hash(body.new.as_bytes()).as_bytes()));
    
    Ok(Json(CommitRefResult { status: "accepted".into(), link_hash }))
}
