mod app;
mod auth;
mod cache;
mod config;
mod db;
mod error;
mod extract;
mod handlers;
#[cfg(feature = "lambda")]
mod lambda;
mod models;
mod ratelimit;
mod widgets;

#[cfg(test)]
mod tests;

use axum::Router;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::app::AppState;
use crate::{
    cache::Cache,
    config::{Config, RedisRequirement},
    db::Database,
};

/// Server entry point: optional `.env`, then a loopback (by default) listener.
#[cfg(not(feature = "lambda"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env first so RUST_LOG from it reaches the tracing filter. A missing
    // file is fine (variables may come from the environment); a malformed one is not.
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(e) if e.not_found() => {}
        Err(e) => return Err(anyhow::anyhow!("failed to load .env: {e}")),
    }
    init_tracing(true);

    let (addr, app) = startup(RedisRequirement::Required).await?;

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

/// Lambda entry point (M6 decision 3): the process environment only (no `.env`), plain-text
/// logs for CloudWatch, and the cache disabled when `REDIS_URL` is unset (decision 9).
#[cfg(feature = "lambda")]
#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    init_tracing(false);
    let (_, app) = startup(RedisRequirement::Optional).await?;
    lambda::run(app).await
}

fn init_tracing(ansi: bool) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "insightboard_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(ansi))
        .init();
}

/// Startup shared by both entry points: configuration, the Postgres pool, migrations, the cache,
/// the application state, and the router. Returns the configured bind address (unused by Lambda)
/// and the router.
async fn startup(redis: RedisRequirement) -> anyhow::Result<(SocketAddr, Router)> {
    let config = Config::from_env(redis)?;

    let addr = SocketAddr::new(config.app_host, config.app_port);
    tracing::info!(
        "Starting InsightBoard backend {}",
        env!("CARGO_PKG_VERSION")
    );
    if let Some(dir) = &config.static_dir {
        tracing::info!("Serving the built frontend from {}", dir.display());
    }

    let db = Database::new(&config.database_url, config.db_max_connections).await?;
    tracing::info!("Database connection established");

    // Ensure schema is present (concurrent starts serialise on sqlx's migration lock).
    db.migrate().await?;
    tracing::info!("Database migrations applied");

    // Redis is optional at startup: an unreachable Redis is a cache miss until it answers.
    let cache = match &config.redis_url {
        Some(url) => Cache::new(url, cache::KEY_PREFIX).await?,
        None => {
            tracing::info!("REDIS_URL is unset: provider-response caching is disabled");
            Cache::disabled(cache::KEY_PREFIX)
        }
    };

    let state = AppState::new(config, db, cache)?;
    state.limiters.spawn_cleanup();
    Ok((addr, app::router(state)))
}

/// Graceful shutdown signal handler
#[cfg(not(feature = "lambda"))]
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
