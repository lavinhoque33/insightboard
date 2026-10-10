//! Application state and the HTTP router, shared by `main` and the router tests.

use axum::{
    extract::Request,
    http::{header, HeaderValue, Method},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::{sync::Arc, time::Duration};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    auth::dummy_password_hash,
    cache::Cache,
    config::Config,
    db::Database,
    error::AppError,
    handlers,
    ratelimit::{self, RateLimiters},
    widgets,
};

/// Outbound HTTP clients, built once.
#[derive(Clone)]
pub struct HttpClients {
    /// Provider widgets (GitHub, OpenWeather, NewsAPI, CoinGecko).
    pub provider: reqwest::Client,
    /// Status widget: policy resolver, no redirects, no proxy.
    pub status: reqwest::Client,
}

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub cache: Cache,
    pub config: Arc<Config>,
    pub http: HttpClients,
    pub limiters: RateLimiters,
    /// Argon2 hash verified against for unknown login emails (timing equalisation).
    pub dummy_hash: Arc<str>,
}

impl AppState {
    pub fn new(config: Config, db: Database, cache: Cache) -> anyhow::Result<Self> {
        let http = HttpClients {
            provider: widgets::http::provider_client(
                config.provider_connect_timeout,
                config.provider_timeout,
            )?,
            status: widgets::status::status_client(
                config.status_connect_timeout,
                config.status_timeout,
            )?,
        };
        let limiters = RateLimiters::new(config.auth_rate_limit, config.data_rate_limit);
        Ok(Self {
            db,
            cache,
            config: Arc::new(config),
            http,
            limiters,
            dummy_hash: Arc::from(dummy_password_hash()?),
        })
    }
}

/// The full application router with middleware and state.
pub fn router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            ratelimit::limit_auth,
        ));

    let data_routes = Router::new()
        .route("/github", get(widgets::github::fetch_github_data))
        .route("/weather", get(widgets::weather::fetch_weather_data))
        .route("/news", get(widgets::news::fetch_news_data))
        .route("/crypto", get(widgets::crypto::fetch_crypto_data))
        .route("/status", get(widgets::status::fetch_status_data))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            ratelimit::limit_data,
        ));

    let api_routes = Router::new()
        .nest("/auth", auth_routes)
        .route("/me", get(handlers::auth::me))
        .route(
            "/dashboards",
            get(handlers::dashboard::list_dashboards).post(handlers::dashboard::create_dashboard),
        )
        .route(
            "/dashboards/:id",
            get(handlers::dashboard::get_dashboard)
                .put(handlers::dashboard::update_dashboard)
                .delete(handlers::dashboard::delete_dashboard),
        )
        .nest("/data", data_routes);

    let cors_origins = state.config.cors_origins.clone();
    let request_timeout = state.config.request_timeout;
    with_middleware(
        Router::new()
            .route("/healthz", get(handlers::health::health_check))
            .route("/readyz", get(handlers::health::readiness_check))
            .nest("/api", api_routes),
        cors_origins,
        request_timeout,
    )
    .with_state(state)
}

async fn not_found() -> Response {
    AppError::NotFound("Not found").into_response()
}

async fn method_not_allowed() -> Response {
    AppError::MethodNotAllowed.into_response()
}

/// Ends any request still running after `limit` with 504 `{error}`.
async fn request_timeout(limit: Duration, req: Request, next: Next) -> Response {
    match tokio::time::timeout(limit, next.run(req)).await {
        Ok(response) => response,
        Err(_) => {
            tracing::warn!(?limit, "request timed out");
            AppError::Timeout.into_response()
        }
    }
}

/// Wrap a router in the HTTP middleware stack (JSON fallbacks, request timeout, CORS allowlist,
/// compression, tracing).
pub fn with_middleware<S>(
    router: Router<S>,
    cors_origins: Vec<HeaderValue>,
    timeout: Duration,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Only what the API uses; no credentials. Foreign origins get no CORS headers.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(cors_origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    router
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .layer(middleware::from_fn(move |req: Request, next: Next| {
            request_timeout(timeout, req, next)
        }))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
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
    use std::time::Duration;
    use tower::ServiceExt;

    const ALLOWED: &str = "http://127.0.0.1:5173";

    fn app() -> Router {
        with_middleware(
            Router::new()
                .route("/api/ping", get(|| async { "pong" }))
                .route(
                    "/api/slow",
                    get(|| async {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        "late"
                    }),
                ),
            vec![HeaderValue::from_static(ALLOWED)],
            Duration::from_millis(100),
        )
    }

    async fn send(req: Request<Body>) -> (StatusCode, serde_json::Value) {
        let res = app().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or_default())
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

    #[tokio::test]
    async fn slow_requests_end_with_a_json_504() {
        let (status, body) = send(
            Request::builder()
                .uri("/api/slow")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
        assert!(body["error"].is_string());
    }

    #[tokio::test]
    async fn unknown_routes_and_methods_are_json() {
        let (status, body) = send(
            Request::builder()
                .uri("/api/nope")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body["error"].is_string());

        let (status, body) = send(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/ping")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert!(body["error"].is_string());
    }
}
