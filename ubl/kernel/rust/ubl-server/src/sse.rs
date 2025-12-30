//! SSE tail endpoint - simplified to emit only cid:seq
//! Reset: payload MINÚSCULO para evitar limite 8KB do PostgreSQL NOTIFY
//!
//! Emits only: "container_id:sequence" (ex: "repo://tenant/ws:42")
//! Client fetches full entry via GET /ledger/:container_id/entry/:sequence if needed

use axum::{routing::get, response::sse::{Sse, Event}, Router};
use futures_util::stream::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use std::pin::Pin;

#[derive(Clone)]
pub struct TailBus {
    pub tx: broadcast::Sender<(String, String)>, // (container_id, sequence_str)
}

impl TailBus {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1024);
        Self { tx }
    }
    
    pub fn notify(&self, container_id: String, sequence: i64) {
        let _ = self.tx.send((container_id, sequence.to_string()));
    }
    
    pub fn stream(&self) -> Pin<Box<dyn Stream<Item = Result<Event, std::convert::Infallible>> + Send>> {
        let rx = self.tx.subscribe();
        let s = BroadcastStream::new(rx).map(|msg| {
            let (cid, seq) = msg.unwrap_or_else(|_| ("_".into(), "0".into()));
            Ok(Event::default().event("entry").data(format!("{cid}:{seq}")))
        });
        Box::pin(s)
    }
}

pub fn sse_router(bus: TailBus) -> Router {
    Router::new().route("/ledger/tail", get({
        let bus = bus.clone();
        move || async move {
            let stream = bus.stream();
            Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
        }
    }))
}
