//! Router-level tests through `tower::ServiceExt::oneshot`. Nothing here reaches the network
//! beyond loopback: providers are `mockito` servers or local listeners on 127.0.0.1, the
//! status widget only sees IP literals, `localhost`, or allowlisted loopback fixtures, and
//! Postgres/Redis are the compose services on 127.0.0.1.

mod auth;
mod cache;
mod contract;
mod dashboards;
mod errors;
mod health;
mod providers;
mod rate_limit;
mod status;

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, Method, Request, StatusCode},
    Router,
};
use reqwest::Url;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{net::SocketAddr, num::NonZeroU32, time::Duration};
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    app::{router, AppState},
    auth::generate_token,
    cache::Cache,
    config::{Config, ProviderUrls, RateLimit},
    db::Database,
};

pub const TEST_SECRET: &str = "router-test-secret-at-least-32-bytes-long";
/// Loopback port 9 (discard): nothing listens, so an unexpected provider call fails fast and
/// locally instead of reaching a real provider.
const CLOSED: &str = "http://127.0.0.1:9";
/// Loopback port 1: a Redis URL that refuses connections (cache fails open).
pub const NO_REDIS: &str = "redis://127.0.0.1:1";

fn rate(per_minute: u32, burst: u32) -> RateLimit {
    RateLimit {
        per_minute: NonZeroU32::new(per_minute).unwrap(),
        burst: NonZeroU32::new(burst).unwrap(),
    }
}

/// A config with closed provider URLs, generous rate limits, and no provider keys.
pub fn test_config() -> Config {
    let closed = Url::parse(CLOSED).unwrap();
    Config {
        app_host: [127, 0, 0, 1].into(),
        app_port: 0,
        database_url: String::new(),
        redis_url: NO_REDIS.to_string(),
        jwt_secret: TEST_SECRET.to_string(),
        cors_origins: vec![],
        github_api_token: None,
        openweather_api_key: None,
        newsapi_api_key: None,
        provider_urls: ProviderUrls {
            github: closed.clone(),
            openweather: closed.clone(),
            newsapi: closed.clone(),
            coingecko: closed,
        },
        provider_connect_timeout: Duration::from_secs(3),
        provider_timeout: Duration::from_secs(8),
        status_connect_timeout: Duration::from_secs(2),
        status_timeout: Duration::from_secs(4),
        status_allowed_fixtures: vec![],
        request_timeout: Duration::from_secs(10),
        auth_rate_limit: rate(10_000, 10_000),
        data_rate_limit: rate(10_000, 10_000),
    }
}

/// The compose Redis URL from the environment (the Makefile exports `.env`; a direct
/// `cargo test` loads the repository `.env` without printing it).
pub fn redis_url() -> String {
    let _ = dotenvy::dotenv();
    std::env::var("REDIS_URL").expect("REDIS_URL must be set (compose Redis; see .env.example)")
}

/// A pool that never connects (for tests that do not touch Postgres).
pub fn unused_pool() -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(500))
        .connect_lazy("postgres://unused:unused@127.0.0.1:1/unused")
        .unwrap()
}

pub struct TestApp {
    pub router: Router,
}

pub struct TestResponse {
    pub status: StatusCode,
    pub body: Value,
    pub content_type: Option<String>,
}

impl TestApp {
    /// A router over `pool` with a fresh cache key prefix (fail-open when Redis is the closed
    /// default).
    pub async fn new(config: Config, pool: PgPool) -> Self {
        let cache = Cache::new(&config.redis_url, &format!("ib-test:{}:", Uuid::new_v4()))
            .await
            .unwrap();
        let state = AppState::new(config, Database::from_pool(pool), cache).unwrap();
        Self {
            router: router(state),
        }
    }

    /// A router with the default test config and no database.
    pub async fn without_db() -> Self {
        Self::new(test_config(), unused_pool()).await
    }

    pub fn token(&self, user_id: Uuid) -> String {
        generate_token(user_id, "user@example.com", TEST_SECRET).unwrap()
    }

    pub async fn send(&self, req: Request<Body>) -> TestResponse {
        let mut req = req;
        if req.extensions().get::<ConnectInfo<SocketAddr>>().is_none() {
            req.extensions_mut()
                .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 40_000))));
        }
        let res = self.router.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let content_type = res
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| panic!("non-JSON body: {}", String::from_utf8_lossy(&bytes)))
        };
        TestResponse {
            status,
            body,
            content_type,
        }
    }

    pub async fn get(&self, uri: &str, token: Option<&str>) -> TestResponse {
        self.send(build(Method::GET, uri, token, None)).await
    }

    pub async fn json(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Value,
    ) -> TestResponse {
        self.send(build(method, uri, token, Some(body))).await
    }
}

pub fn build(method: Method, uri: &str, token: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

/// Insert a user directly (no Argon2) and return its id.
pub async fn insert_user(pool: &PgPool, email: &str) -> Uuid {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash) VALUES ($1, 'not-a-real-hash') RETURNING id",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .unwrap();
    id
}

/// A loopback listener that accepts connections and never answers (a slow upstream). The
/// accept task ends with the test's runtime.
pub async fn silent_server() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            held.push(stream);
        }
    });
    addr
}

/// A loopback port with nothing listening.
pub async fn closed_port() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap()
}
