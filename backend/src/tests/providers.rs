//! Provider widgets against local `mockito` servers and a silent loopback listener.

use axum::http::StatusCode;
use mockito::Matcher;
use reqwest::Url;
use serde_json::json;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::{
    contract::{assert_def, assert_error_body, assert_list_of},
    silent_server, test_config, TestApp,
};
use crate::{config::Config, widgets::http::USER_AGENT};

const KEY: &str = "test-provider-key-0123456789";

async fn app_with(config: Config) -> (TestApp, String) {
    let app = TestApp::new(config, super::unused_pool()).await;
    let token = app.token(Uuid::new_v4());
    (app, token)
}

fn url(server: &mockito::ServerGuard) -> Url {
    Url::parse(&server.url()).unwrap()
}

#[tokio::test]
async fn github_sends_user_agent_token_and_matches_the_contract() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/users/octocat/events/public")
        .match_header("user-agent", USER_AGENT)
        .match_header("authorization", "Bearer gh-test-token")
        .with_header("content-type", "application/json")
        .with_body(
            json!([{
                "id": "1", "type": "PushEvent", "repo": {"name": "octocat/hello"},
                "created_at": "2026-10-10T10:00:00Z", "payload": {"size": 1}, "public": true
            }])
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let mut config = test_config();
    config.provider_urls.github = url(&server);
    config.github_api_token = Some("gh-test-token".into());
    let (app, token) = app_with(config).await;

    // Mixed case is normalised (one account, one cache entry).
    let res = app
        .get("/api/data/github?username=OctoCat", Some(&token))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_list_of("GitHubEvent", &res.body);
    mock.assert_async().await;
}

#[tokio::test]
async fn github_rejects_path_like_usernames_without_calling_upstream() {
    let mut server = mockito::Server::new_async().await;
    let any = server
        .mock("GET", Matcher::Any)
        .expect(0)
        .create_async()
        .await;
    let mut config = test_config();
    config.provider_urls.github = url(&server);
    let (app, token) = app_with(config).await;

    for username in ["a%2F..%2Fb", "..", "a/b", "repos%2Fx", "-a", "a--b", ""] {
        let res = app
            .get(
                &format!("/api/data/github?username={username}"),
                Some(&token),
            )
            .await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{username}");
        assert_error_body(res.status, &res.body);
        assert_eq!(res.body["details"][0]["field"], "username");
    }
    any.assert_async().await;
}

#[tokio::test]
async fn weather_encodes_the_city_and_matches_the_contract() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/data/2.5/weather")
        .match_header("user-agent", USER_AGENT)
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("q".into(), "São Paulo/../x?&y=1".into()),
            Matcher::UrlEncoded("appid".into(), KEY.into()),
            Matcher::UrlEncoded("units".into(), "metric".into()),
        ]))
        .with_body(
            json!({
                "name": "São Paulo", "sys": {"country": "BR"},
                "main": {"temp": 21.5, "feels_like": 21.0, "humidity": 70},
                "weather": [{"description": "clear sky", "icon": "01d"}],
                "wind": {"speed": 3.2}
            })
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let mut config = test_config();
    config.provider_urls.openweather = url(&server);
    config.openweather_api_key = Some(KEY.into());
    let (app, token) = app_with(config).await;

    let city = "S%C3%A3o%20Paulo%2F..%2Fx%3F%26y%3D1";
    let res = app
        .get(&format!("/api/data/weather?city={city}"), Some(&token))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_def("WeatherData", &res.body);
    mock.assert_async().await;
}

#[tokio::test]
async fn news_key_travels_in_the_header_not_the_url() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v2/everything")
        .match_header("x-api-key", KEY)
        .match_header("user-agent", USER_AGENT)
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("q".into(), "rust & go".into()),
            Matcher::UrlEncoded("pageSize".into(), "10".into()),
        ]))
        .match_request(|req| {
            let pq = req.path_and_query();
            !pq.contains(KEY) && !pq.to_ascii_lowercase().contains("apikey")
        })
        .with_body(
            json!({"status": "ok", "articles": [
                {"title": "A", "description": null, "url": "https://example.com/a",
                 "source": {"name": "Ex"}, "publishedAt": "2026-10-10T10:00:00Z", "urlToImage": null},
                {"title": "dropped: no url", "publishedAt": "2026-10-10T10:00:00Z", "source": {"name": "Ex"}}
            ]})
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let mut config = test_config();
    config.provider_urls.newsapi = url(&server);
    config.newsapi_api_key = Some(KEY.into());
    let (app, token) = app_with(config).await;

    let res = app
        .get("/api/data/news?topic=rust%20%26%20go", Some(&token))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_list_of("NewsArticle", &res.body);
    assert_eq!(res.body.as_array().unwrap().len(), 1);
    mock.assert_async().await;
}

#[tokio::test]
async fn crypto_keeps_null_numbers_and_normalises_ids() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/v3/coins/markets")
        .match_header("user-agent", USER_AGENT)
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("ids".into(), "bitcoin,new-coin".into()),
            Matcher::UrlEncoded("vs_currency".into(), "usd".into()),
        ]))
        .with_body(
            json!([
                {"id": "bitcoin", "symbol": "btc", "name": "Bitcoin", "current_price": 60000.5,
                 "price_change_percentage_24h": -1.2, "market_cap": 1.2e12, "total_volume": 3.4e10},
                {"id": "new-coin", "symbol": "new", "name": "New Coin", "current_price": null,
                 "price_change_percentage_24h": null, "market_cap": null},
                {"id": "nameless", "symbol": "nl", "name": null, "current_price": 1.0}
            ])
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let mut config = test_config();
    config.provider_urls.coingecko = url(&server);
    let (app, token) = app_with(config).await;

    let res = app
        .get(
            "/api/data/crypto?ids=%20New-Coin%20,BITCOIN,bitcoin",
            Some(&token),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_list_of("CryptoPrice", &res.body);
    let items = res.body.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert!(items[1]["current_price"].is_null());
    assert!(items[1]["total_volume"].is_null());
    mock.assert_async().await;
}

#[tokio::test]
async fn upstream_errors_are_a_generic_502() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", Matcher::Any)
        .with_status(500)
        .with_body(format!("internal failure; key={KEY}"))
        .create_async()
        .await;
    let mut config = test_config();
    let base = url(&server);
    config.provider_urls.github = base.clone();
    config.provider_urls.openweather = base.clone();
    config.provider_urls.newsapi = base.clone();
    config.provider_urls.coingecko = base;
    config.openweather_api_key = Some(KEY.into());
    config.newsapi_api_key = Some(KEY.into());
    let (app, token) = app_with(config).await;

    for uri in [
        "/api/data/github?username=octocat",
        "/api/data/weather?city=Paris",
        "/api/data/news",
        "/api/data/crypto",
    ] {
        let res = app.get(uri, Some(&token)).await;
        assert_eq!(res.status, StatusCode::BAD_GATEWAY, "{uri}");
        assert_eq!(
            res.body,
            json!({"error": "External service error"}),
            "{uri}"
        );
    }

    // Undecodable success bodies are the same generic 502.
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", Matcher::Any)
        .with_body("<html>not json</html>")
        .create_async()
        .await;
    let mut config = test_config();
    config.provider_urls.coingecko = url(&server);
    let (app, token) = app_with(config).await;
    let res = app.get("/api/data/crypto", Some(&token)).await;
    assert_eq!(res.status, StatusCode::BAD_GATEWAY);
    assert_eq!(res.body, json!({"error": "External service error"}));
}

/// Drill 7: every provider against a slow local upstream ends at the configured timeout.
#[tokio::test]
async fn slow_upstreams_end_at_the_configured_timeout() {
    let silent = Url::parse(&format!("http://{}", silent_server().await)).unwrap();
    let mut config = test_config();
    config.provider_timeout = Duration::from_millis(300);
    config.provider_urls.github = silent.clone();
    config.provider_urls.openweather = silent.clone();
    config.provider_urls.newsapi = silent.clone();
    config.provider_urls.coingecko = silent;
    config.openweather_api_key = Some(KEY.into());
    config.newsapi_api_key = Some(KEY.into());
    let (app, token) = app_with(config).await;

    for uri in [
        "/api/data/github?username=octocat",
        "/api/data/weather?city=Paris",
        "/api/data/news",
        "/api/data/crypto",
    ] {
        let start = Instant::now();
        let res = app.get(uri, Some(&token)).await;
        let elapsed = start.elapsed();
        assert_eq!(res.status, StatusCode::BAD_GATEWAY, "{uri}");
        assert_eq!(res.body, json!({"error": "External service error"}));
        assert!(
            elapsed >= Duration::from_millis(250) && elapsed < Duration::from_secs(3),
            "{uri} took {elapsed:?}"
        );
    }
}

#[tokio::test]
async fn missing_provider_keys_are_503() {
    let (app, token) = app_with(test_config()).await;
    for uri in ["/api/data/weather?city=Paris", "/api/data/news"] {
        let res = app.get(uri, Some(&token)).await;
        assert_eq!(res.status, StatusCode::SERVICE_UNAVAILABLE, "{uri}");
        assert_error_body(res.status, &res.body);
    }
}

#[tokio::test]
async fn missing_or_invalid_query_input_is_422_with_the_parameter_name() {
    let (app, token) = app_with(test_config()).await;
    for (uri, field) in [
        ("/api/data/github", "username"),
        ("/api/data/weather", "city"),
        ("/api/data/weather?city=%20%20", "city"),
        ("/api/data/news?topic=%20", "topic"),
        ("/api/data/crypto?ids=,,", "ids"),
        ("/api/data/status", "urls"),
    ] {
        let res = app.get(uri, Some(&token)).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{uri}");
        assert_error_body(res.status, &res.body);
        assert_eq!(res.body["details"][0]["field"], field, "{uri}");
    }
}
