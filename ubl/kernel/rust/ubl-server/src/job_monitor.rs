//! Job Monitor - Diamond Checklist #8
//!
//! Background worker that monitors jobs for timeouts and marks orphaned jobs.
//! An orphaned job is one that's stuck in 'in_progress' with no recent activity.

use sqlx::{PgPool, Row};
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error};

/// Configuration for job monitoring
#[derive(Clone)]
pub struct JobMonitorConfig {
    /// How often to check for orphaned jobs (in seconds)
    pub check_interval_secs: u64,
    /// How long before a job is considered orphaned (in minutes)
    pub timeout_minutes: i64,
}

impl Default for JobMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60, // Check every minute
            timeout_minutes: 30,     // 30 minutes without activity = orphaned
        }
    }
}

/// Job Monitor - monitors for orphaned jobs
pub struct JobMonitor {
    pool: PgPool,
    config: JobMonitorConfig,
}

impl JobMonitor {
    pub fn new(pool: PgPool, config: JobMonitorConfig) -> Self {
        Self { pool, config }
    }

    /// Start the monitoring loop (runs forever)
    pub async fn run(self) {
        info!(
            "🔍 Job Monitor started - checking every {}s for jobs idle > {}m",
            self.config.check_interval_secs, self.config.timeout_minutes
        );

        let mut tick = interval(Duration::from_secs(self.config.check_interval_secs));

        loop {
            tick.tick().await;
            
            if let Err(e) = self.check_orphaned_jobs().await {
                error!("❌ Job monitor error: {}", e);
            }
        }
    }

    /// Check for orphaned jobs and mark them as timed out
    async fn check_orphaned_jobs(&self) -> Result<(), sqlx::Error> {
        // Diamond Checklist #8: Find jobs stuck in 'in_progress' with no recent activity
        let result = sqlx::query(
            r#"
            UPDATE projection_jobs
            SET state = 'failed',
                updated_at = NOW(),
                last_event_hash = 'timeout_monitor',
                last_event_seq = last_event_seq
            WHERE state = 'in_progress'
              AND (last_activity_at IS NULL OR last_activity_at < NOW() - INTERVAL '1 minute' * $1)
            RETURNING job_id, title, last_activity_at
            "#
        )
        .bind(self.config.timeout_minutes)
        .fetch_all(&self.pool)
        .await?;

        if !result.is_empty() {
            for row in &result {
                let job_id: String = row.try_get("job_id").unwrap_or_default();
                let title: String = row.try_get("title").unwrap_or_default();
                warn!(
                    "⏱️  Job orphaned: {} ({}) - marked as failed",
                    job_id, title
                );
            }
            info!("🔧 Marked {} orphaned jobs as 'failed'", result.len());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = JobMonitorConfig::default();
        assert_eq!(config.check_interval_secs, 60);
        assert_eq!(config.timeout_minutes, 30);
    }
}
