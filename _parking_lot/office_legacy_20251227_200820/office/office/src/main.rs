//! OFFICE Server - LLM Operating System HTTP API
//!
//! Provides HTTP/WebSocket API for managing LLM entities and sessions.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use office::{OfficeConfig, Result};
use office::api::{create_router, AppState};
use office::ubl_client::UblClient;
use office::llm::create_provider;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .json()
        .init();

    info!("Starting OFFICE - LLM Operating System");

    // Load configuration
    let config = load_config()?;
    info!("Configuration loaded: {:?}", config.server);

    // Initialize UBL client with generated signing key
    let ubl_client = Arc::new(UblClient::with_generated_key(
        &config.ubl.endpoint,
        &config.ubl.container_id,
        config.ubl.timeout_ms,
    ));
    info!("UBL client initialized: {}", config.ubl.endpoint);

    // Initialize LLM provider
    let llm_provider = create_provider(&config.llm)?;
    info!("LLM provider initialized: {}", config.llm.provider);

    // Create application state
    let state = AppState::new(config.clone(), ubl_client, llm_provider);
    let shared_state = Arc::new(RwLock::new(state));

    // Create router
    let app = create_router(shared_state);

    // Bind and serve
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("OFFICE server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn load_config() -> Result<OfficeConfig> {
    // Try to load from environment or file, fallback to defaults
    let config = config::Config::builder()
        .add_source(config::File::with_name("config/development").required(false))
        .add_source(config::Environment::with_prefix("OFFICE").separator("__"))
        .build()
        .ok()
        .and_then(|c| c.try_deserialize().ok())
        .unwrap_or_else(OfficeConfig::default);

    Ok(config)
}
