//! Drill 8: auth (per client IP) and data (per user) limits answer 429 `{error}`.

use axum::{
    extract::ConnectInfo,
    http::{Method, StatusCode},
};
use serde_json::json;
use std::{net::SocketAddr, num::NonZeroU32};
use uuid::Uuid;

use super::{build, contract::assert_error_body, test_config, unused_pool, TestApp};
use crate::config::RateLimit;

fn tiny() -> RateLimit {
    RateLimit {
        per_minute: NonZeroU32::new(1).unwrap(),
        burst: NonZeroU32::new(2).unwrap(),
    }
}

#[tokio::test]
async fn auth_routes_are_limited_per_client_ip() {
    let mut config = test_config();
    config.auth_rate_limit = tiny();
    let app = TestApp::new(config, unused_pool()).await;

    let login = |ip: [u8; 4]| {
        let mut req = build(Method::POST, "/api/auth/login", None, Some(json!({})));
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from((ip, 50_000))));
        req
    };
    // Invalid bodies (422) still count: the limit runs before the handler.
    assert_eq!(
        app.send(login([10, 1, 1, 1])).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let register = {
        let mut req = build(Method::POST, "/api/auth/register", None, Some(json!({})));
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([10, 1, 1, 1], 50_001))));
        req
    };
    assert_eq!(
        app.send(register).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let limited = app.send(login([10, 1, 1, 1])).await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
    assert_error_body(limited.status, &limited.body);

    // Another client is unaffected.
    assert_eq!(
        app.send(login([10, 2, 2, 2])).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn data_routes_are_limited_per_user_after_authentication() {
    let mut config = test_config();
    config.data_rate_limit = tiny();
    let app = TestApp::new(config, unused_pool()).await;
    let alice = app.token(Uuid::new_v4());
    let bob = app.token(Uuid::new_v4());
    let uri = "/api/data/status";

    for _ in 0..2 {
        let res = app.get(uri, Some(&alice)).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    }
    let limited = app.get(uri, Some(&alice)).await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
    assert_error_body(limited.status, &limited.body);

    // Other users keep their own budget; unauthenticated requests are 401, not 429.
    assert_eq!(
        app.get(uri, Some(&bob)).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for _ in 0..3 {
        assert_eq!(app.get(uri, None).await.status, StatusCode::UNAUTHORIZED);
    }
}
