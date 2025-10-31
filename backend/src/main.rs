mod auth;
mod cache;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod widgets;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    db::Database,
    cache::Cache,
};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub cache: Cache,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "insightboard_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Starting InsightBoard backend on port {}", config.app_port);
    let app_port = config.app_port; // Save port before moving config

    // Initialize database connection
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connection established");

    // Initialize Redis cache
    let cache = Cache::new(&config.redis_url).await?;
    tracing::info!("Redis cache connection established");

    // Create application state
    let state = AppState {
        db,
        cache,
        config,
    };

    // Build the router
    let app = Router::new()
        // Health check endpoint
        .route("/healthz", get(handlers::health::health_check))
        
        // API routes
        .nest("/api", api_routes())
        
        // Middleware
        .layer(CorsLayer::permissive()) // Configure CORS properly in production
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        
        // Share state
        .with_state(state);

    // Bind to address
    let addr = SocketAddr::from(([0, 0, 0, 0], app_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    tracing::info!("Listening on {}", addr);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// API routes organization
fn api_routes() -> Router<AppState> {
    Router::new()
        // Authentication routes
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/me", get(handlers::auth::me))
        
        // Dashboard routes (protected)
        .route("/dashboards", get(handlers::dashboard::list_dashboards))
        .route("/dashboards", post(handlers::dashboard::create_dashboard))
        .route("/dashboards/:id", get(handlers::dashboard::get_dashboard))
        .route("/dashboards/:id", put(handlers::dashboard::update_dashboard))
        .route("/dashboards/:id", delete(handlers::dashboard::delete_dashboard))
        
        // Widget data routes (protected)
        .route("/data/github", get(widgets::github::fetch_github_data))
        .route("/data/weather", get(widgets::weather::fetch_weather_data))
        .route("/data/news", get(widgets::news::fetch_news_data))
        .route("/data/crypto", get(widgets::crypto::fetch_crypto_data))
        .route("/data/status", get(widgets::status::fetch_status_data))
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
