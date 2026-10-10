//! M5 section 1: the built frontend (`STATIC_DIR`) served beside the API, with the SPA
//! fallback, cache and security headers, and `/api` paths that never fall back to `index.html`.

use axum::{
    body::Body,
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use serde_json::Value;
use std::path::PathBuf;
use tower::ServiceExt;
use uuid::Uuid;

use super::{
    contract::{assert_def, assert_error_body},
    test_config, unused_pool, TestApp,
};

const INDEX: &str = "<!doctype html><html><body><div id=\"app\"></div></body></html>";
const SCRIPT: &str = "console.log('insightboard frontend bundle, long enough to be compressed');";
const SECRET: &str = "outside the static directory";
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// A throwaway build directory (`index.html`, a hashed asset, a public file) with a file
/// next to it that must never be served. Removed on drop.
struct Dist {
    root: PathBuf,
}

impl Dist {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("ib-static-{}", Uuid::new_v4()));
        let dist = root.join("dist");
        std::fs::create_dir_all(dist.join("assets")).unwrap();
        std::fs::write(dist.join("index.html"), INDEX).unwrap();
        std::fs::write(
            dist.join("favicon.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
        )
        .unwrap();
        std::fs::write(dist.join("assets/index-AbC123.js"), SCRIPT).unwrap();
        std::fs::write(root.join("secret.txt"), SECRET).unwrap();
        Self { root }
    }

    fn dir(&self) -> PathBuf {
        self.root.join("dist")
    }
}

impl Drop for Dist {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct Raw {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl Raw {
    fn header(&self, name: header::HeaderName) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    fn text(&self) -> &str {
        std::str::from_utf8(&self.body).unwrap()
    }

    fn json(&self) -> Value {
        assert_eq!(self.header(header::CONTENT_TYPE), Some("application/json"));
        serde_json::from_slice(&self.body).unwrap()
    }

    fn assert_security_headers(&self) {
        assert_eq!(self.header(header::X_CONTENT_TYPE_OPTIONS), Some("nosniff"));
        assert_eq!(self.header(header::REFERRER_POLICY), Some("no-referrer"));
        assert_eq!(self.header(header::X_FRAME_OPTIONS), Some("DENY"));
    }
}

async fn app_with(static_dir: Option<PathBuf>) -> TestApp {
    let mut config = test_config();
    config.static_dir = static_dir;
    TestApp::new(config, unused_pool()).await
}

async fn send(app: &TestApp, method: Method, uri: &str, gzip: bool) -> Raw {
    let mut req = Request::builder().method(method).uri(uri);
    if gzip {
        req = req.header(header::ACCEPT_ENCODING, "gzip");
    }
    let res = app
        .router
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    Raw {
        status,
        headers,
        body,
    }
}

async fn get(app: &TestApp, uri: &str) -> Raw {
    send(app, Method::GET, uri, false).await
}

#[tokio::test]
async fn hashed_assets_are_served_immutable_with_security_headers_and_compression() {
    let dist = Dist::new();
    let app = app_with(Some(dist.dir())).await;

    let res = get(&app, "/assets/index-AbC123.js").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.text(), SCRIPT);
    assert!(res
        .header(header::CONTENT_TYPE)
        .is_some_and(|ct| ct.contains("javascript")));
    assert_eq!(res.header(header::CACHE_CONTROL), Some(IMMUTABLE));
    res.assert_security_headers();

    let res = send(&app, Method::GET, "/assets/index-AbC123.js", true).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.header(header::CONTENT_ENCODING), Some("gzip"));
    assert_eq!(res.header(header::CACHE_CONTROL), Some(IMMUTABLE));
}

#[tokio::test]
async fn root_public_files_and_deep_links_get_revalidated_html() {
    let dist = Dist::new();
    let app = app_with(Some(dist.dir())).await;

    for uri in [
        "/",
        "/index.html",
        "/login",
        "/dashboards/3f1c2a9e-1b2c-4d5e-8f90-123456789abc",
        "/dashboards/3f1c2a9e-1b2c-4d5e-8f90-123456789abc/",
    ] {
        let res = get(&app, uri).await;
        assert_eq!(res.status, StatusCode::OK, "{uri}");
        assert_eq!(res.text(), INDEX, "{uri}");
        assert!(
            res.header(header::CONTENT_TYPE)
                .is_some_and(|ct| ct.starts_with("text/html")),
            "{uri}"
        );
        assert_eq!(res.header(header::CACHE_CONTROL), Some("no-cache"), "{uri}");
        res.assert_security_headers();
    }

    let res = get(&app, "/favicon.svg").await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.text().starts_with("<svg"));
    assert_eq!(res.header(header::CACHE_CONTROL), Some("no-cache"));

    // HEAD works like GET without a body.
    let res = send(&app, Method::HEAD, "/dashboards/abc", false).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.body.is_empty());
}

#[tokio::test]
async fn api_paths_and_missing_assets_never_fall_back_to_index_html() {
    let dist = Dist::new();
    let app = app_with(Some(dist.dir())).await;

    for (method, uri) in [
        (Method::GET, "/api/unknown"),
        (Method::GET, "/api"),
        (Method::GET, "/api/"),
        (Method::GET, "/api/dashboards/x/y"),
        (Method::POST, "/api/unknown"),
        (Method::GET, "/assets/missing-123.js"),
    ] {
        let res = send(&app, method.clone(), uri, false).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{method} {uri}");
        assert_error_body(res.status, &res.json());
        assert_ne!(res.header(header::CACHE_CONTROL), Some(IMMUTABLE), "{uri}");
    }
}

#[tokio::test]
async fn files_outside_the_static_directory_are_never_served() {
    let dist = Dist::new();
    let app = app_with(Some(dist.dir())).await;
    for uri in [
        "/../secret.txt",
        "/%2e%2e/secret.txt",
        "/assets/../../secret.txt",
        "/assets/%2e%2e/%2e%2e/secret.txt",
    ] {
        let res = get(&app, uri).await;
        assert!(!res.text().contains(SECRET), "{uri}");
    }
}

#[tokio::test]
async fn unservable_file_names_are_treated_as_missing_files() {
    let dist = Dist::new();
    let app = app_with(Some(dist.dir())).await;
    let long = "a".repeat(256);
    let encoded_long = "%61".repeat(200);

    for segment in [long.as_str(), encoded_long.as_str(), "x%00y", "%00"] {
        // Elsewhere: the SPA fallback, as for any missing file.
        for uri in [format!("/{segment}"), format!("/dashboards/{segment}")] {
            let res = get(&app, &uri).await;
            assert_eq!(res.status, StatusCode::OK, "{uri}");
            assert_eq!(res.text(), INDEX, "{uri}");
            assert_eq!(res.header(header::CACHE_CONTROL), Some("no-cache"), "{uri}");
            res.assert_security_headers();
        }

        // Under /assets/: a JSON 404 that is never cached as immutable.
        let uri = format!("/assets/{segment}");
        let res = get(&app, &uri).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{uri}");
        assert_error_body(res.status, &res.json());
        assert_ne!(res.header(header::CACHE_CONTROL), Some(IMMUTABLE), "{uri}");
    }

    // 255 bytes is a legal file name length: still an ordinary miss.
    let res = get(&app, &format!("/{}", "a".repeat(255))).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.text(), INDEX);
}

#[tokio::test]
async fn health_api_and_method_handling_are_unchanged_with_a_static_dir() {
    let dist = Dist::new();
    let app = app_with(Some(dist.dir())).await;

    let res = get(&app, "/healthz").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_def("HealthResponse", &res.json());

    // No Postgres or Redis behind this router: readiness is a JSON 503, not the SPA.
    let res = get(&app, "/readyz").await;
    assert_eq!(res.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_error_body(res.status, &res.json());

    let res = get(&app, "/api/dashboards").await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_error_body(res.status, &res.json());

    // Wrong methods stay a JSON 405, for API routes and for frontend paths alike.
    for (method, uri) in [
        (Method::PATCH, "/api/dashboards"),
        (Method::POST, "/healthz"),
        (Method::POST, "/dashboards/abc"),
        (Method::DELETE, "/assets/index-AbC123.js"),
    ] {
        let res = send(&app, method.clone(), uri, false).await;
        assert_eq!(res.status, StatusCode::METHOD_NOT_ALLOWED, "{method} {uri}");
        assert_error_body(res.status, &res.json());
    }
}

#[tokio::test]
async fn without_a_static_dir_every_unknown_path_is_a_json_404() {
    let app = app_with(None).await;
    for uri in [
        "/",
        "/index.html",
        "/dashboards/abc",
        "/assets/index-AbC123.js",
        "/api/unknown",
    ] {
        let res = get(&app, uri).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{uri}");
        assert_error_body(res.status, &res.json());
        assert!(!res.headers.contains_key(header::X_FRAME_OPTIONS), "{uri}");
    }
    let res = get(&app, "/healthz").await;
    assert_eq!(res.status, StatusCode::OK);
}
