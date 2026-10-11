//! M6 section 1: Function URL / HTTP API payload-v2 events, parsed by `lambda_http` itself,
//! through the Lambda adapter layer and the application router, in-process.

use axum::{
    http::{header, StatusCode},
    Router,
};
use serde_json::{json, Value};
use std::num::NonZeroU32;
use tower::ServiceExt;

use super::{
    contract::{assert_def, assert_error_body},
    frontend::{Dist, INDEX, SCRIPT},
    test_config, unused_pool, TestApp,
};
use crate::{config::RateLimit, lambda::with_source_ip};

const HOST: &str = "abcdefghijklmnopqrstuvwxyz012345.lambda-url.eu-west-1.on.aws";

/// A Function URL event (payload format 2.0). `source_ip: None` omits `sourceIp`. Every event
/// carries the same `X-Forwarded-For`, which must never influence the client address.
fn event(method: &str, path: &str, source_ip: Option<&str>, body: Option<Value>) -> String {
    let mut http = json!({
        "method": method,
        "path": path,
        "protocol": "HTTP/1.1",
        "userAgent": "insightboard-test"
    });
    if let Some(ip) = source_ip {
        http["sourceIp"] = json!(ip);
    }
    let mut event = json!({
        "version": "2.0",
        "routeKey": "$default",
        "rawPath": path,
        "rawQueryString": "",
        "headers": {
            "host": HOST,
            "user-agent": "insightboard-test",
            "x-forwarded-for": "203.0.113.250",
            "x-forwarded-proto": "https"
        },
        "requestContext": {
            "accountId": "anonymous",
            "apiId": "abcdefghijklmnopqrstuvwxyz012345",
            "domainName": HOST,
            "domainPrefix": "abcdefghijklmnopqrstuvwxyz012345",
            "http": http,
            "requestId": "c6af9ac6-7b61-11e6-9a41-93e8deadbeef",
            "routeKey": "$default",
            "stage": "$default",
            "time": "10/Oct/2026:12:00:00 +0000",
            "timeEpoch": 1791633600000u64
        },
        "isBase64Encoded": false
    });
    if let Some(body) = body {
        event["headers"]["content-type"] = json!("application/json");
        event["body"] = json!(body.to_string());
    }
    event.to_string()
}

struct LambdaResponse {
    status: StatusCode,
    content_type: Option<String>,
    body: Vec<u8>,
}

impl LambdaResponse {
    fn json(&self) -> Value {
        assert_eq!(self.content_type.as_deref(), Some("application/json"));
        serde_json::from_slice(&self.body).unwrap()
    }

    fn text(&self) -> &str {
        std::str::from_utf8(&self.body).unwrap()
    }
}

/// Parse `event` with `lambda_http` and send it through the adapter layer and `router`.
async fn invoke(router: &Router, event: &str) -> LambdaResponse {
    let req = lambda_http::request::from_str(event).expect("a valid payload-v2 event");
    let res = with_source_ip(router.clone()).oneshot(req).await.unwrap();
    let status = res.status();
    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    LambdaResponse {
        status,
        content_type,
        body,
    }
}

#[tokio::test]
async fn healthz_static_files_deep_links_and_api_404_work_through_the_adapter() {
    let dist = Dist::new();
    let mut config = test_config();
    config.static_dir = Some(dist.dir());
    let app = TestApp::new(config, unused_pool()).await;
    let ip = Some("198.51.100.7");

    let health = invoke(&app.router, &event("GET", "/healthz", ip, None)).await;
    assert_eq!(health.status, StatusCode::OK);
    let body = health.json();
    assert_def("HealthResponse", &body);
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));

    let asset = invoke(
        &app.router,
        &event("GET", "/assets/index-AbC123.js", ip, None),
    )
    .await;
    assert_eq!(asset.status, StatusCode::OK);
    assert!(asset
        .content_type
        .as_deref()
        .is_some_and(|ct| ct.contains("javascript")));
    assert_eq!(asset.text(), SCRIPT);

    for path in ["/", "/dashboards/3f2b0c1e-1111-4222-8333-944445555666"] {
        let page = invoke(&app.router, &event("GET", path, ip, None)).await;
        assert_eq!(page.status, StatusCode::OK, "{path}");
        assert!(
            page.content_type
                .as_deref()
                .is_some_and(|ct| ct.starts_with("text/html")),
            "{path}"
        );
        assert_eq!(page.text(), INDEX, "{path}");
    }

    let unknown = invoke(&app.router, &event("GET", "/api/unknown", ip, None)).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_error_body(unknown.status, &unknown.json());
}

/// An app whose auth limit allows a burst of two requests per client.
async fn tightly_limited_app() -> TestApp {
    let mut config = test_config();
    config.auth_rate_limit = RateLimit {
        per_minute: NonZeroU32::new(1).unwrap(),
        burst: NonZeroU32::new(2).unwrap(),
    };
    TestApp::new(config, unused_pool()).await
}

/// Invalid bodies (422) still count against the limit, so no database is needed.
async fn auth_status(app: &TestApp, route: &str, source_ip: Option<&str>) -> StatusCode {
    let path = format!("/api/auth/{route}");
    invoke(
        &app.router,
        &event("POST", &path, source_ip, Some(json!({}))),
    )
    .await
    .status
}

#[tokio::test]
async fn register_and_login_are_limited_per_source_ip() {
    let app = tightly_limited_app().await;
    let first = Some("198.51.100.7");
    let second = Some("2001:db8::7");

    assert_eq!(
        auth_status(&app, "register", first).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        auth_status(&app, "login", first).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        auth_status(&app, "login", first).await,
        StatusCode::TOO_MANY_REQUESTS
    );

    // Same X-Forwarded-For, different source address: a separate bucket.
    assert_eq!(
        auth_status(&app, "register", second).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        auth_status(&app, "login", second).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn events_without_a_usable_source_ip_share_one_bucket() {
    let app = tightly_limited_app().await;

    // Missing and unparseable source addresses fall back to the shared bucket.
    assert_eq!(
        auth_status(&app, "register", None).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        auth_status(&app, "login", Some("not-an-ip")).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        auth_status(&app, "login", None).await,
        StatusCode::TOO_MANY_REQUESTS
    );

    // A caller with a real source address keeps its own budget.
    assert_eq!(
        auth_status(&app, "login", Some("198.51.100.8")).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
