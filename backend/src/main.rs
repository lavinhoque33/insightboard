mod auth;
mod cache;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod widgets;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{cache::Cache, config::Config, db::Database};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub cache: Cache,
    pub config: Config,
}

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

    let addr = SocketAddr::new(config.app_host, config.app_port); // Save before moving config
    tracing::info!("Starting InsightBoard backend on {}", addr);

    // Initialize database connection
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connection established");

    // Ensure schema is present in local/dev runs.
    db.migrate().await?;
    tracing::info!("Database migrations applied");

    // Initialize Redis cache
    let cache = Cache::new(&config.redis_url).await?;
    tracing::info!("Redis cache connection established");

    // Create application state
    let state = AppState { db, cache, config };

    // Build the router
    let cors_origins = state.config.cors_origins.clone();
    let app = with_middleware(
        Router::new()
            // Health check endpoint
            .route("/healthz", get(handlers::health::health_check))
            // API routes
            .nest("/api", api_routes()),
        cors_origins,
    )
    // Share state
    .with_state(state);

    // Bind to address
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Listening on {}", addr);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Wrap a router in the HTTP middleware stack (CORS allowlist, compression, tracing).
fn with_middleware<S>(router: Router<S>, cors_origins: Vec<HeaderValue>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Only what the API uses; no credentials. Foreign origins get no CORS headers.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(cors_origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    router
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
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
        .route(
            "/dashboards/:id",
            put(handlers::dashboard::update_dashboard),
        )
        .route(
            "/dashboards/:id",
            delete(handlers::dashboard::delete_dashboard),
        )
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

#[cfg(test)]
mod tests {
    use super::with_middleware;
    use axum::{
        body::Body,
        http::{header, HeaderValue, Method, Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    const ALLOWED: &str = "http://127.0.0.1:5173";

    fn app() -> Router {
        with_middleware(
            Router::new().route("/api/ping", get(|| async { "pong" })),
            vec![HeaderValue::from_static(ALLOWED)],
        )
    }

    async fn preflight(origin: &str) -> axum::response::Response {
        app()
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/api/ping")
                    .header(header::ORIGIN, origin)
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn preflight_from_allowed_origin_is_granted() {
        let res = preflight(ALLOWED).await;
        assert_eq!(
            res.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&HeaderValue::from_static(ALLOWED))
        );
        assert!(res
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_METHODS));
        assert!(!res
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS));
    }

    #[tokio::test]
    async fn preflight_from_foreign_origin_gets_no_grant() {
        let res = preflight("https://evil.example").await;
        assert!(!res
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    }

    #[tokio::test]
    async fn simple_request_from_foreign_origin_gets_no_grant() {
        let res = app()
            .oneshot(
                Request::builder()
                    .uri("/api/ping")
                    .header(header::ORIGIN, "https://evil.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(!res
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    }
}
