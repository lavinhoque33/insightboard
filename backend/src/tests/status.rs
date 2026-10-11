//! Status widget policy through the router (drill 5). Targets are IP literals, `localhost`, or
//! loopback fixtures only; blocked entries never produce a connection.

use axum::http::StatusCode;
use mockito::Matcher;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::{
    closed_port,
    contract::{assert_error_body, assert_list_of},
    silent_server, test_config, TestApp,
};
use crate::config::Config;

fn query(urls: &[String]) -> String {
    let joined = urls.join(",");
    let encoded: String = url_encode(&joined);
    format!("/api/data/status?urls={encoded}")
}

fn url_encode(raw: &str) -> String {
    let mut url = reqwest::Url::parse("http://x/").unwrap();
    url.query_pairs_mut().append_pair("v", raw);
    url.query().unwrap().trim_start_matches("v=").to_string()
}

async fn app_with(config: Config) -> (TestApp, String) {
    let app = TestApp::new(config, super::unused_pool()).await;
    let token = app.token(Uuid::new_v4());
    (app, token)
}

fn errors(body: &Value) -> Vec<Value> {
    body.as_array()
        .unwrap()
        .iter()
        .map(|c| c["error"].clone())
        .collect()
}

#[tokio::test]
async fn rejected_targets_get_generic_classes_and_no_request_leaves() {
    let mut fixture = mockito::Server::new_async().await;
    let never = fixture
        .mock("GET", Matcher::Any)
        .expect(0)
        .create_async()
        .await;
    let port = fixture.socket_address().port();
    let (app, token) = app_with(test_config()).await;

    let urls: Vec<String> = vec![
        format!("http://127.0.0.1:{port}/"),
        format!("http://localhost:{port}/"),
        format!("http://[::ffff:127.0.0.1]:{port}/"),
        "http://169.254.169.254/latest/meta-data/".into(),
        "http://[fd00:ec2::254]/".into(),
        "http://10.0.0.1/".into(),
        "http://100.64.0.1/".into(),
        "ftp://example.com/".into(),
        "file:///etc/passwd".into(),
        "not a url".into(),
    ];
    let res = app.get(&query(&urls), Some(&token)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_list_of("StatusCheck", &res.body);
    let blocked = json!("blocked_address");
    let invalid = json!("invalid_url");
    assert_eq!(
        errors(&res.body),
        [
            blocked.clone(),
            blocked.clone(),
            blocked.clone(),
            blocked.clone(),
            blocked.clone(),
            blocked.clone(),
            blocked,
            invalid.clone(),
            invalid.clone(),
            invalid,
        ]
    );
    for check in res.body.as_array().unwrap() {
        assert_eq!(check["status"], 0);
    }
    never.assert_async().await;
}

#[tokio::test]
async fn more_urls_than_the_cap_is_422() {
    let (app, token) = app_with(test_config()).await;
    let urls: Vec<String> = (0..11).map(|i| format!("http://192.0.2.{i}/")).collect();
    let res = app.get(&query(&urls), Some(&token)).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_error_body(res.status, &res.body);
    assert_eq!(res.body["details"][0]["field"], "urls");
}

#[tokio::test]
async fn allowlisted_fixture_is_reached_and_redirects_are_reported_not_followed() {
    let mut fixture = mockito::Server::new_async().await;
    let ok = fixture
        .mock("GET", "/ok")
        .with_status(200)
        .expect(1)
        .create_async()
        .await;
    let redirect = fixture
        .mock("GET", "/redirect")
        .with_status(302)
        .with_header("location", "http://169.254.169.254/")
        .expect(1)
        .create_async()
        .await;
    let mut config = test_config();
    config.status_allowed_fixtures = vec![fixture.socket_address()];
    let (app, token) = app_with(config).await;

    let base = fixture.url();
    let res = app
        .get(
            &query(&[format!("{base}/ok"), format!("{base}/redirect")]),
            Some(&token),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_list_of("StatusCheck", &res.body);
    assert_eq!(res.body[0]["status"], 200);
    assert_eq!(res.body[0]["error"], Value::Null);
    assert_eq!(res.body[1]["status"], 302);
    assert_eq!(res.body[1]["error"], Value::Null);
    ok.assert_async().await;
    redirect.assert_async().await;
}

#[tokio::test]
async fn timeouts_and_refused_connections_are_classified() {
    let silent = silent_server().await;
    let closed = closed_port().await;
    let mut config = test_config();
    config.status_timeout = Duration::from_millis(300);
    config.status_allowed_fixtures = vec![silent, closed];
    let (app, token) = app_with(config).await;

    let res = app
        .get(
            &query(&[format!("http://{silent}/"), format!("http://{closed}/")]),
            Some(&token),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_list_of("StatusCheck", &res.body);
    assert_eq!(
        errors(&res.body),
        [json!("timeout"), json!("connect_error")]
    );
}

#[tokio::test]
async fn checks_run_five_at_a_time() {
    let silent = silent_server().await;
    let mut config = test_config();
    config.status_timeout = Duration::from_millis(400);
    config.status_allowed_fixtures = vec![silent];
    let (app, token) = app_with(config).await;

    let urls: Vec<String> = (0..10).map(|i| format!("http://{silent}/{i}")).collect();
    let start = Instant::now();
    let res = app.get(&query(&urls), Some(&token)).await;
    let elapsed = start.elapsed();
    assert_eq!(res.status, StatusCode::OK);
    assert!(errors(&res.body).iter().all(|e| e == "timeout"));
    // Two waves of five: about 2 x 400 ms; sequential would be 4 s, unbounded 400 ms.
    assert!(
        elapsed >= Duration::from_millis(750) && elapsed < Duration::from_millis(2_500),
        "took {elapsed:?}"
    );
}

#[tokio::test]
async fn results_are_cached_per_user() {
    let mut fixture = mockito::Server::new_async().await;
    let hits = fixture.mock("GET", "/up").expect(2).create_async().await;
    let mut config = test_config();
    config.redis_url = Some(super::redis_url());
    config.status_allowed_fixtures = vec![fixture.socket_address()];
    let app = TestApp::new(config, super::unused_pool()).await;
    let alice = app.token(Uuid::new_v4());
    let bob = app.token(Uuid::new_v4());
    let uri = query(&[format!("{}/up", fixture.url())]);

    for token in [&alice, &alice, &bob, &bob] {
        let res = app.get(&uri, Some(token)).await;
        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(res.body[0]["status"], 200);
    }
    // One upstream hit per user: the second request of each was served from that user's cache.
    hits.assert_async().await;
}
