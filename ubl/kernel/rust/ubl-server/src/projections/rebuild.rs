//! Projection rebuild from ledger events
//!
//! This module handles rebuilding projections from the ledger.
//! Used on startup or to repair corrupted projections.

use sqlx::PgPool;
use tracing::{info, error};
use super::{JobsProjection, MessagesProjection};

/// Rebuild all projections from the ledger
pub async fn rebuild_projections(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("🔄 Starting projection rebuild...");

    let jobs = JobsProjection::new(pool.clone());
    let messages = MessagesProjection::new(pool.clone());

    // Get all atoms ordered by container and sequence
    let atoms = sqlx::query!(
        r#"
        SELECT 
            la.container_id,
            la.atom_hash,
            la.atom_data,
            le.entry_hash,
            le.sequence
        FROM ledger_atom la
        JOIN ledger_entry le ON le.link_hash = la.atom_hash
        ORDER BY le.container_id, le.sequence ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut jobs_count = 0;
    let mut messages_count = 0;

    for atom in atoms {
        let event_type = atom.atom_data["type"].as_str().unwrap_or("");
        
        if atom.container_id == "C.Jobs" {
            if let Err(e) = jobs.process_event(
                event_type,
                &atom.atom_data,
                &atom.entry_hash,
                atom.sequence,
            ).await {
                error!("Failed to process job event: {}", e);
            }
            jobs_count += 1;
        } else if atom.container_id == "C.Messenger" {
            if let Err(e) = messages.process_event(
                event_type,
                &atom.atom_data,
                &atom.entry_hash,
                atom.sequence,
            ).await {
                error!("Failed to process message event: {}", e);
            }
            messages_count += 1;
        }
    }

    // Update projection state
    for container_id in ["C.Jobs", "C.Messenger"] {
        if let Ok(Some(last)) = sqlx::query!(
            r#"
            SELECT sequence, entry_hash
            FROM ledger_entry
            WHERE container_id = $1
            ORDER BY sequence DESC
            LIMIT 1
            "#,
            container_id
        )
        .fetch_optional(pool)
        .await
        {
            sqlx::query!(
                r#"
                UPDATE projection_state
                SET last_sequence = $2, last_hash = $3, last_rebuild = NOW()
                WHERE container_id = $1
                "#,
                container_id,
                last.sequence,
                last.entry_hash
            )
            .execute(pool)
            .await?;
        }
    }

    info!(
        "✅ Projection rebuild complete: {} job events, {} message events",
        jobs_count, messages_count
    );

    Ok(())
}

