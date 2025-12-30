//! Job Artifacts Projection
//!
//! Tracks artifacts produced by jobs from tool.result events.

use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::info;

/// Artifacts projection handler
pub struct ArtifactsProjection {
    pool: PgPool,
}

impl ArtifactsProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Process tool.result event and extract artifacts
    pub async fn process_event(
        &self,
        atom: &serde_json::Value,
        entry_hash: &str,
        tenant_id: &str,
    ) -> Result<(), sqlx::Error> {
        let job_id = atom.get("job_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if job_id.is_empty() {
            return Ok(());
        }

        // Extract artifacts from tool.result payload
        let empty_vec: Vec<serde_json::Value> = vec![];
        let artifacts = atom.get("payload")
            .and_then(|p| p.get("artifacts"))
            .and_then(|a| a.as_array())
            .unwrap_or(&empty_vec);

        for artifact in artifacts {
            if let Some(artifact_id) = artifact.get("artifact_id").and_then(|v| v.as_str()) {
                let kind = artifact.get("kind").and_then(|v| v.as_str()).unwrap_or("file");
                let title = artifact.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
                let url = artifact.get("url").and_then(|v| v.as_str());
                let mime_type = artifact.get("mime_type").and_then(|v| v.as_str());
                let size_bytes = artifact.get("size_bytes").and_then(|v| v.as_u64());

                sqlx::query!(
                    r#"
                    INSERT INTO projection_job_artifacts (
                        tenant_id, job_id, artifact_id, kind, title,
                        url, mime_type, size_bytes, event_id, created_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    ON CONFLICT (tenant_id, job_id, artifact_id) DO UPDATE SET
                        url = EXCLUDED.url,
                        mime_type = EXCLUDED.mime_type,
                        size_bytes = EXCLUDED.size_bytes
                    "#,
                    tenant_id,
                    job_id,
                    artifact_id,
                    kind,
                    title,
                    url,
                    mime_type,
                    size_bytes.map(|s| s as i64),
                    entry_hash,
                    OffsetDateTime::now_utc()
                )
                .execute(&self.pool)
                .await?;

                info!("📦 Artifact stored: {} {} ({})", job_id, artifact_id, kind);
            }
        }

        Ok(())
    }

    /// Get artifacts for a job
    pub async fn get_artifacts(
        &self,
        tenant_id: &str,
        job_id: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT artifact_id, kind, title, url, mime_type, size_bytes, created_at
            FROM projection_job_artifacts
            WHERE tenant_id = $1 AND job_id = $2
            ORDER BY created_at DESC
            "#,
            tenant_id,
            job_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| {
            serde_json::json!({
                "artifact_id": r.artifact_id,
                "kind": r.kind,
                "title": r.title,
                "url": r.url,
                "mime_type": r.mime_type,
                "size_bytes": r.size_bytes,
                "created_at": r.created_at.to_string(),
            })
        }).collect())
    }
}

