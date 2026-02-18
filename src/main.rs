use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

mod api;
mod backend;
mod config;
mod error;
mod streaming;
mod validation;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "kokoro_openai_server=info,axum=info".to_string());
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    info!("Starting Kokoro OpenAI Server v{}", env!("CARGO_PKG_VERSION"));

    // Parse configuration
    let config = Config::from_env_and_args().context("Failed to parse configuration")?;
    
    info!("Configuration loaded:");
    info!("  Host: {}:{}", config.host, config.port);
    info!("  Workers: {}", config.workers);
    info!("  Max input chars: {}", config.max_input_chars);
    info!("  Execution provider: {:?}", config.execution_provider);
    
    if config.api_key.is_some() {
        info!("  Authentication: enabled");
    } else {
        warn!("  Authentication: disabled (set API_KEY to enable)");
    }

    // Initialize backend
    let backend = backend::KokoroBackend::new(&config)
        .await
        .context("Failed to initialize Kokoro backend")?;
    
    info!("Backend initialized successfully");

    // Build router
    let app = api::create_router(Arc::new(backend), config.api_key.clone(), config.max_input_chars);

    // Create socket address
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .context("Invalid host:port combination")?;

    info!("Server listening on http://{}", addr);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    info!("Server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}