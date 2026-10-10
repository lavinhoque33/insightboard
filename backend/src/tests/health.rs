//! D19: `/healthz` is dependency-free; `/readyz` reflects Postgres and Redis.

use axum::http::StatusCode;
use sqlx::PgPool;

use super::{
    contract::{assert_def, assert_error_body},
    redis_url, test_config, unused_pool, TestApp,
};

#[tokio::test]
async fn healthz_needs_no_dependency_and_readyz_reports_them_down() {
    // Neither Postgres (closed port) nor Redis (closed port) is reachable.
    let app = TestApp::without_db().await;
    let health = app.get("/healthz", None).await;
    assert_eq!(health.status, StatusCode::OK);
    assert_def("HealthResponse", &health.body);

    let ready = app.get("/readyz", None).await;
    assert_eq!(ready.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_error_body(ready.status, &ready.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn readyz_is_200_when_postgres_and_redis_answer(pool: PgPool) {
    let mut config = test_config();
    config.redis_url = redis_url();
    let app = TestApp::new(config, pool).await;
    let ready = app.get("/readyz", None).await;
    assert_eq!(ready.status, StatusCode::OK);
    assert_def("ReadyResponse", &ready.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn readyz_is_503_when_only_redis_is_down(pool: PgPool) {
    let app = TestApp::new(test_config(), pool).await;
    let ready = app.get("/readyz", None).await;
    assert_eq!(ready.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_error_body(ready.status, &ready.body);
}

#[tokio::test]
async fn readyz_is_503_when_only_postgres_is_down() {
    let mut config = test_config();
    config.redis_url = redis_url();
    let app = TestApp::new(config, unused_pool()).await;
    let ready = app.get("/readyz", None).await;
    assert_eq!(ready.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_error_body(ready.status, &ready.body);
}
