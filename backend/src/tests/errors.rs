//! D16: extractor rejections and every other failure render as the JSON error model.

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use uuid::Uuid;

use super::{contract::assert_error_body, TestApp};

fn raw(
    method: Method,
    uri: &str,
    token: Option<&str>,
    content_type: Option<&str>,
    body: &str,
) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(ct) = content_type {
        builder = builder.header(header::CONTENT_TYPE, ct);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

#[tokio::test]
async fn body_rejections_are_json_with_the_agreed_statuses() {
    let app = TestApp::without_db().await;
    let token = app.token(Uuid::new_v4());
    let dashboard = format!("/api/dashboards/{}", Uuid::new_v4());
    let json = Some("application/json");

    let cases = [
        // Malformed JSON → 400.
        (
            Method::POST,
            "/api/auth/register",
            None,
            json,
            "{\"email\":",
            StatusCode::BAD_REQUEST,
        ),
        (
            Method::PUT,
            dashboard.as_str(),
            Some(token.as_str()),
            json,
            "not json",
            StatusCode::BAD_REQUEST,
        ),
        // Missing or wrong content type → 415.
        (
            Method::POST,
            "/api/auth/login",
            None,
            None,
            "{}",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            Method::POST,
            "/api/dashboards",
            Some(token.as_str()),
            Some("text/plain"),
            "{}",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        // Well-formed but not an object / missing fields → 422 with details.
        (
            Method::POST,
            "/api/auth/register",
            None,
            json,
            "[1,2]",
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            Method::POST,
            "/api/auth/login",
            None,
            json,
            "{}",
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            Method::POST,
            "/api/dashboards",
            Some(token.as_str()),
            json,
            "{\"name\": 1}",
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ];
    for (method, uri, token, content_type, body, expected) in cases {
        let res = app
            .send(raw(method.clone(), uri, token, content_type, body))
            .await;
        assert_eq!(res.status, expected, "{method} {uri} {body:?}");
        assert_eq!(
            res.content_type.as_deref(),
            Some("application/json"),
            "{method} {uri}"
        );
        assert_error_body(res.status, &res.body);
    }
}

#[tokio::test]
async fn unauthorized_and_unknown_routes_are_json() {
    let app = TestApp::without_db().await;
    for uri in ["/api/dashboards", "/api/data/crypto", "/api/me"] {
        let res = app.get(uri, None).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert_error_body(res.status, &res.body);
    }
    let res = app.get("/api/unknown", None).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_error_body(res.status, &res.body);
}

#[tokio::test]
async fn database_unavailable_is_a_generic_500() {
    // The unused pool points at a closed loopback port.
    let app = TestApp::without_db().await;
    let token = app.token(Uuid::new_v4());
    let res = app.get("/api/dashboards", Some(&token)).await;
    assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_error_body(res.status, &res.body);
}
