//! Cache behaviour against compose Redis, each test under its own key prefix.

use axum::http::StatusCode;
use redis::AsyncCommands;
use reqwest::Url;
use serde_json::json;
use uuid::Uuid;

use super::{redis_url, test_config, unused_pool, TestApp};
use crate::cache::Cache;

async fn cache() -> (Cache, String) {
    let prefix = format!("ib-test:{}:", Uuid::new_v4());
    (Cache::new(&redis_url(), &prefix).await.unwrap(), prefix)
}

#[tokio::test]
async fn values_round_trip_with_a_ttl() {
    let (cache, prefix) = cache().await;
    let key = cache.key("weather", &["paris"]);
    assert!(key.starts_with(&prefix));
    assert_eq!(cache.get::<Vec<u32>>(&key).await, None);
    cache.set(&key, &vec![1u32, 2, 3], 30).await;
    assert_eq!(cache.get::<Vec<u32>>(&key).await, Some(vec![1, 2, 3]));

    let client = redis::Client::open(redis_url()).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let ttl: i64 = conn.ttl(&key).await.unwrap();
    assert!((1..=30).contains(&ttl), "ttl {ttl}");
    let _: () = conn.del(&key).await.unwrap();
}

#[tokio::test]
async fn undecodable_entries_are_a_miss() {
    let (cache, _) = cache().await;
    let key = cache.key("crypto", &["bitcoin"]);
    cache.set(&key, &"not a list", 30).await;
    assert_eq!(cache.get::<Vec<u32>>(&key).await, None);
}

#[tokio::test]
async fn provider_data_is_shared_under_normalised_keys() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/v3/coins/markets")
        .match_query(mockito::Matcher::Any)
        .with_body(
            json!([{"id": "bitcoin", "symbol": "btc", "name": "Bitcoin", "current_price": 1.0,
                     "price_change_percentage_24h": 0.0, "market_cap": 1.0, "total_volume": 1.0}])
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let mut config = test_config();
    config.redis_url = redis_url();
    config.provider_urls.coingecko = Url::parse(&server.url()).unwrap();
    let app = TestApp::new(config, unused_pool()).await;
    let alice = app.token(Uuid::new_v4());
    let bob = app.token(Uuid::new_v4());

    // Same set of ids in different order/case/spacing, from two users: one upstream call.
    for (uri, token) in [
        ("/api/data/crypto?ids=bitcoin", &alice),
        ("/api/data/crypto?ids=%20BITCOIN%20,bitcoin", &bob),
    ] {
        let res = app.get(uri, Some(token)).await;
        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(res.body[0]["id"], "bitcoin");
    }
    mock.assert_async().await;
}
