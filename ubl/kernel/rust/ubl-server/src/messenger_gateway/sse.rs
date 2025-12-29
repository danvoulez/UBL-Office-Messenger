//! Gateway SSE Delta Stream
//!
//! Emits SSE deltas to frontend clients.
//! Events: timeline.append, job.update, presence.update, conversation.update

use axum::response::sse::{Event, Sse};
use futures_util::Stream;
use serde::Serialize;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::{debug, error};

/// SSE delta event types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeltaEvent {
    Hello { cursor: String },
    TimelineAppend { conversation_id: String, item: serde_json::Value },
    JobUpdate { job_id: String, update: serde_json::Value },
    PresenceUpdate { entity_id: String, state: String },
    ConversationUpdate { conversation_id: String, update: serde_json::Value },
    Heartbeat,
    Error { message: String },
}

/// SSE stream handler for Gateway
#[derive(Clone)]
pub struct GatewaySSE {
    sender: mpsc::Sender<DeltaEvent>,
}

impl GatewaySSE {
    pub fn new() -> (Self, Sse<impl Stream<Item = Result<Event, Infallible>>>) {
        let (tx, rx) = mpsc::channel(128);
        
        let stream = ReceiverStream::new(rx).map(|event| {
            let json = serde_json::to_string(&event).unwrap_or_default();
            Ok(Event::default()
                .event(event_type_name(&event))
                .data(json))
        });

        let sse = Sse::new(stream)
            .keep_alive(axum::response::sse::KeepAlive::default());

        (Self { sender: tx }, sse)
    }

    pub async fn emit(&self, event: DeltaEvent) -> Result<(), mpsc::error::SendError<DeltaEvent>> {
        self.sender.send(event).await
    }
}

fn event_type_name(event: &DeltaEvent) -> &str {
    match event {
        DeltaEvent::Hello { .. } => "hello",
        DeltaEvent::TimelineAppend { .. } => "timeline.append",
        DeltaEvent::JobUpdate { .. } => "job.update",
        DeltaEvent::PresenceUpdate { .. } => "presence.update",
        DeltaEvent::ConversationUpdate { .. } => "conversation.update",
        DeltaEvent::Heartbeat => "heartbeat",
        DeltaEvent::Error { .. } => "error",
    }
}

