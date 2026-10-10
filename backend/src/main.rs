mod app;
mod auth;
mod cache;
mod config;
mod db;
mod error;
mod extract;
mod handlers;
mod models;
mod ratelimit;
mod widgets;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::app::AppState;
use crate::{cache::Cache, config::Config, db::Database};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env first so RUST_LOG from it reaches the tracing filter. A missing
    // file is fine (variables may come from the environment); a malformed one is not.
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(e) if e.not_found() => {}
        Err(e) => return Err(anyhow::anyhow!("failed to load .env: {e}")),
    }

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "insightboard_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    let addr = SocketAddr::new(config.app_host, config.app_port);
    tracing::info!("Starting InsightBoard backend on {}", addr);

    // Initialize database connection
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connection established");

    // Ensure schema is present in local/dev runs.
    db.migrate().await?;
    tracing::info!("Database migrations applied");

    // Redis is optional at startup: an unreachable Redis is a cache miss until it answers.
    let cache = Cache::new(&config.redis_url, cache::KEY_PREFIX).await?;

    let state = AppState::new(config, db, cache)?;
    state.limiters.spawn_cleanup();
    let app = app::router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", addr);

    // Peer addresses feed the per-IP auth rate limit.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}
