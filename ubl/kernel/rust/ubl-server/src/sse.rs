//! SSE tail endpoint with PostgreSQL LISTEN/NOTIFY
//! PR10: Real-time ledger streaming
//!
//! FIX: Postgres NOTIFY 8KB limit (Gemini P0 #2)
//! - Trigger sends only reference: {container_id, sequence, entry_hash}
//! - SSE handler fetches full payload via SELECT
//! - Supports Last-Event-ID for reconnection (Gemini P2 #7)

use axum::response::sse::{Event, Sse};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::{convert::Infallible, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::{debug, error, warn};

/// Reference from NOTIFY (lightweight, always < 100 bytes)
#[derive(Debug, Deserialize)]
struct NotifyRef {
    container_id: String,
    sequence: i64,
    entry_hash: String,
}

/// Full entry for SSE
#[derive(Debug, Serialize)]
struct LedgerEvent {
    container_id: String,
    sequence: i64,
    entry_hash: String,
    link_hash: String,
    previous_hash: String,
    ts_unix_ms: i64,
    atom: Option<Value>,
}

/// Fetch full ledger entry by container_id and sequence
async fn fetch_entry(pool: &PgPool, container_id: &str, sequence: i64) -> Option<LedgerEvent> {
    let row = sqlx::query(
        r#"
        SELECT 
            e.container_id,
            e.sequence,
            e.entry_hash,
            e.link_hash,
            e.previous_hash,
            e.ts_unix_ms,
            a.data as atom
        FROM ledger_entry e
        LEFT JOIN ledger_atom a ON e.link_hash = a.hash
        WHERE e.container_id = $1 AND e.sequence = $2
        "#,
    )
    .bind(container_id)
    .bind(sequence)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    row.map(|r| LedgerEvent {
        container_id: r.get("container_id"),
        sequence: r.get("sequence"),
        entry_hash: r.get("entry_hash"),
        link_hash: r.get("link_hash"),
        previous_hash: r.get("previous_hash"),
        ts_unix_ms: r.get("ts_unix_ms"),
        atom: r.get("atom"),
    })
}

/// Fetch entries since a given sequence (for reconnection)
async fn fetch_entries_since(
    pool: &PgPool,
    container_id: &str,
    since_sequence: i64,
    limit: i64,
) -> Vec<LedgerEvent> {
    let rows = sqlx::query(
        r#"
        SELECT 
            e.container_id,
            e.sequence,
            e.entry_hash,
            e.link_hash,
            e.previous_hash,
            e.ts_unix_ms,
            a.data as atom
        FROM ledger_entry e
        LEFT JOIN ledger_atom a ON e.link_hash = a.hash
        WHERE e.container_id = $1 AND e.sequence > $2
        ORDER BY e.sequence ASC
        LIMIT $3
        "#,
    )
    .bind(container_id)
    .bind(since_sequence)
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|r| LedgerEvent {
            container_id: r.get("container_id"),
            sequence: r.get("sequence"),
            entry_hash: r.get("entry_hash"),
            link_hash: r.get("link_hash"),
            previous_hash: r.get("previous_hash"),
            ts_unix_ms: r.get("ts_unix_ms"),
            atom: r.get("atom"),
        })
        .collect()
}

/// SSE tail for a specific container
/// Listens to PostgreSQL NOTIFY and streams events
///
/// Supports Last-Event-ID header for reconnection (Gemini P2 #7)
pub async fn sse_tail(
    pool: PgPool,
    container_id: String,
    last_event_id: Option<i64>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<LedgerEvent>(128);
    let pool_clone = pool.clone();
    let container_id_clone = container_id.clone();

    // If Last-Event-ID provided, send missed events first
    if let Some(since_seq) = last_event_id {
        let tx_replay = tx.clone();
        let pool_replay = pool.clone();
        let cid_replay = container_id.clone();

        tokio::spawn(async move {
            let missed = fetch_entries_since(&pool_replay, &cid_replay, since_seq, 1000).await;
            if !missed.is_empty() {
                debug!(
                    "📤 Replaying {} missed events for {} since seq {}",
                    missed.len(),
                    cid_replay,
                    since_seq
                );
                for entry in missed {
                    if tx_replay.send(entry).await.is_err() {
                        break;
                    }
                }
            }
        });
    }

    // Spawn task to listen to PostgreSQL NOTIFY
    tokio::spawn(async move {
        match sqlx::postgres::PgListener::connect_with(&pool_clone).await {
            Ok(mut listener) => {
                if let Err(e) = listener.listen("ledger_events").await {
                    error!("Failed to LISTEN on ledger_events: {}", e);
                    return;
                }

                debug!("🔊 LISTEN ledger_events for container: {}", container_id_clone);

                // Process notifications
                loop {
                    match listener.recv().await {
                        Ok(notification) => {
                            let payload = notification.payload();

                            // Parse lightweight reference
                            match serde_json::from_str::<NotifyRef>(payload) {
                                Ok(ref_data) => {
                                    // Filter by container_id
                                    if ref_data.container_id != container_id_clone {
                                        continue;
                                    }

                                    // Fetch full entry
                                    if let Some(entry) = fetch_entry(
                                        &pool_clone,
                                        &ref_data.container_id,
                                        ref_data.sequence,
                                    )
                                    .await
                                    {
                                        debug!(
                                            "📨 SSE event for {}: seq={}",
                                            container_id_clone, entry.sequence
                                        );

                                        if tx.send(entry).await.is_err() {
                                            debug!("SSE client disconnected");
                                            break;
                                        }
                                    } else {
                                        warn!(
                                            "Could not fetch entry: {} seq={}",
                                            ref_data.container_id, ref_data.sequence
                                        );
                                    }
                                }
                                Err(e) => {
                                    warn!("Invalid NOTIFY payload: {} - {}", payload, e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("NOTIFY error: {}", e);
                            tokio::time::sleep(Duration::from_millis(200)).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create PgListener: {}", e);
            }
        }
    });

    // Convert mpsc channel to SSE stream with event ID
    let stream = ReceiverStream::new(rx).map(|entry| {
        let id = entry.sequence.to_string();
        let json = serde_json::to_string(&entry).unwrap_or_default();

        Ok(Event::default()
            .event("ledger_entry")
            .id(id) // For Last-Event-ID reconnection
            .data(json))
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
