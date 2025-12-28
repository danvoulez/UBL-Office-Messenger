//! SSE tail endpoint with PostgreSQL LISTEN/NOTIFY
//! PR10: Real-time ledger streaming

use axum::response::sse::{Event, Sse};
use futures_util::Stream;
use serde_json::Value;
use sqlx::PgPool;
use std::{convert::Infallible, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::{debug, error};

/// SSE tail for a specific container
/// Listens to PostgreSQL NOTIFY and streams only events for the requested container
pub async fn sse_tail(
    pool: PgPool,
    container_id: String,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<String>(128);

    // Spawn task to listen to PostgreSQL NOTIFY
    tokio::spawn(async move {
        match sqlx::postgres::PgListener::connect_with(&pool).await {
            Ok(mut listener) => {
                if let Err(e) = listener.listen("ledger_events").await {
                    error!("Failed to LISTEN on ledger_events: {}", e);
                    return;
                }

                debug!("🔊 LISTEN ledger_events for container: {}", container_id);

                // Process notifications
                loop {
                    match listener.recv().await {
                        Ok(notification) => {
                            let payload = notification.payload().to_string();
                            
                            // Parse JSON and filter by container_id
                            if let Ok(v) = serde_json::from_str::<Value>(&payload) {
                                if let Some(cid) = v.get("container_id").and_then(|x| x.as_str()) {
                                    if cid == container_id.as_str() {
                                        debug!("📨 SSE event for {}: {}", container_id, &payload[..100.min(payload.len())]);
                                        
                                        // Send to SSE stream
                                        if tx.send(payload).await.is_err() {
                                            debug!("SSE client disconnected");
                                            break;
                                        }
                                    }
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

    // Convert mpsc channel to SSE stream
    let stream = ReceiverStream::new(rx).map(|json| {
        Ok(Event::default()
            .event("ledger_entry")
            .data(json))
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
