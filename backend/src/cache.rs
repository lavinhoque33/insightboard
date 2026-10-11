use redis::{
    aio::{ConnectionManager, ConnectionManagerConfig},
    AsyncCommands, Client,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    future::Future,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

/// Key prefix for every key the backend writes.
pub const KEY_PREFIX: &str = "insightboard:";
/// Bound on a single connection attempt.
const CONNECT_TIMEOUT: Duration = Duration::from_millis(1_000);
/// Bound on a single command's response.
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(500);
/// Outer bound on any cache operation, including waiting for a reconnect.
const OPERATION_TIMEOUT: Duration = Duration::from_millis(1_500);
/// After a failed connect, requests skip Redis (a miss) for this long instead of retrying.
const RECONNECT_BACKOFF: Duration = Duration::from_secs(5);
/// Key parts longer than this, or with characters outside `[a-z0-9._,-]`, are hashed.
const MAX_RAW_KEY_PART: usize = 100;

#[derive(Default)]
struct ConnState {
    conn: Option<ConnectionManager>,
    last_failure: Option<Instant>,
}

/// Redis cache. Every operation is bounded and fails open: an unavailable or slow Redis is a
/// cache miss with a warning, never a request failure (D19). A disabled cache (no Redis; the
/// Lambda entry point without `REDIS_URL`, M6 decision 9) never connects: reads miss and writes
/// are skipped silently.
#[derive(Clone)]
pub struct Cache {
    /// `None` when the cache is disabled.
    client: Option<Client>,
    prefix: Arc<str>,
    state: Arc<Mutex<ConnState>>,
}

impl Cache {
    /// Open the client and try a first connection. Only an invalid URL is fatal; an unreachable
    /// Redis is logged and retried on later requests.
    pub async fn new(redis_url: &str, prefix: &str) -> anyhow::Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| anyhow::anyhow!("REDIS_URL is not a valid Redis URL: {e}"))?;
        let cache = Self {
            client: Some(client),
            prefix: Arc::from(prefix),
            state: Arc::new(Mutex::new(ConnState::default())),
        };
        if let Err(e) = cache.ping().await {
            tracing::warn!(error = %e, "Redis unavailable at startup; caching disabled until it answers");
        }
        Ok(cache)
    }

    /// A cache without Redis: no connection is ever attempted.
    pub fn disabled(prefix: &str) -> Self {
        Self {
            client: None,
            prefix: Arc::from(prefix),
            state: Arc::new(Mutex::new(ConnState::default())),
        }
    }

    /// `false` for [`Cache::disabled`]; readiness then checks Postgres only.
    pub fn is_enabled(&self) -> bool {
        self.client.is_some()
    }

    async fn connection(&self) -> Result<ConnectionManager, String> {
        let Some(client) = &self.client else {
            return Err("cache disabled (no REDIS_URL)".to_string());
        };
        let mut state = self.state.lock().await;
        if let Some(conn) = &state.conn {
            return Ok(conn.clone());
        }
        if state
            .last_failure
            .is_some_and(|at| at.elapsed() < RECONNECT_BACKOFF)
        {
            return Err("Redis connection backing off after a failure".to_string());
        }
        let config = ConnectionManagerConfig::new()
            .set_connection_timeout(CONNECT_TIMEOUT)
            .set_response_timeout(RESPONSE_TIMEOUT)
            .set_number_of_retries(1);
        let attempt = tokio::time::timeout(
            OPERATION_TIMEOUT,
            ConnectionManager::new_with_config(client.clone(), config),
        )
        .await;
        match attempt {
            Ok(Ok(conn)) => {
                state.conn = Some(conn.clone());
                state.last_failure = None;
                Ok(conn)
            }
            Ok(Err(e)) => {
                state.last_failure = Some(Instant::now());
                Err(format!("connect failed: {e}"))
            }
            Err(_) => {
                state.last_failure = Some(Instant::now());
                Err("connect timed out".to_string())
            }
        }
    }

    /// Run one command with the outer timeout.
    async fn run<T, F, Fut>(&self, op: F) -> Result<T, String>
    where
        F: FnOnce(ConnectionManager) -> Fut,
        Fut: Future<Output = redis::RedisResult<T>>,
    {
        let conn = self.connection().await?;
        match tokio::time::timeout(OPERATION_TIMEOUT, op(conn)).await {
            Ok(result) => result.map_err(|e| e.to_string()),
            Err(_) => Err("operation timed out".to_string()),
        }
    }

    /// Build a namespaced, bounded key: `<prefix><namespace>:<part>:<part>...`.
    pub fn key(&self, namespace: &str, parts: &[&str]) -> String {
        let mut key = format!("{}{namespace}", self.prefix);
        for part in parts {
            key.push(':');
            key.push_str(&bounded_part(part));
        }
        key
    }

    /// Get a value; any failure (unavailable, timeout, undecodable) is a miss, and a disabled
    /// cache always misses.
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        if !self.is_enabled() {
            return None;
        }
        let owned = key.to_string();
        let raw: Option<String> = match self
            .run(|mut conn| async move { conn.get(owned).await })
            .await
        {
            Ok(raw) => raw,
            Err(e) => {
                tracing::warn!(error = %e, "cache get failed; treating as a miss");
                return None;
            }
        };
        match serde_json::from_str(&raw?) {
            Ok(value) => Some(value),
            Err(e) => {
                tracing::warn!(error = %e, "cached value undecodable; treating as a miss");
                None
            }
        }
    }

    /// Set a value with a TTL in seconds; a failure is logged and ignored, and a disabled cache
    /// skips it.
    pub async fn set<T>(&self, key: &str, value: &T, ttl_secs: u64)
    where
        T: Serialize,
    {
        if !self.is_enabled() {
            return;
        }
        let serialized = match serde_json::to_string(value) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "cache value not serialisable; skipping set");
                return;
            }
        };
        let owned = key.to_string();
        if let Err(e) =
            self.run(|mut conn| async move {
                conn.set_ex::<_, _, ()>(owned, serialized, ttl_secs).await
            })
            .await
        {
            tracing::warn!(error = %e, "cache set failed; continuing without caching");
        }
    }

    /// Readiness probe: `PING` within the operation timeout.
    pub async fn ping(&self) -> Result<(), String> {
        self.run(|mut conn| async move {
            redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await
                .map(|_| ())
        })
        .await
    }
}

/// A key part as-is when short and plain; otherwise `h` + a 128-bit hex digest.
fn bounded_part(part: &str) -> String {
    let plain = part.len() <= MAX_RAW_KEY_PART
        && part
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._,-".contains(c));
    if plain {
        return part.to_string();
    }
    let digest = |seed: u8| {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        part.hash(&mut hasher);
        hasher.finish()
    };
    format!("h{:016x}{:016x}", digest(0), digest(1))
}

#[cfg(test)]
mod tests {
    use super::{bounded_part, Cache};
    use std::time::{Duration, Instant};

    #[test]
    fn plain_short_parts_stay_readable() {
        assert_eq!(bounded_part("bitcoin,ethereum"), "bitcoin,ethereum");
        assert_eq!(bounded_part("new-york"), "new-york");
    }

    #[test]
    fn long_or_special_parts_are_hashed_and_bounded() {
        let long = "a".repeat(5_000);
        for part in [long.as_str(), "New York", "a:b", "x\ny", "ü"] {
            let bounded = bounded_part(part);
            assert_eq!(bounded.len(), 33, "{part:?}");
            assert!(bounded.starts_with('h'));
        }
        assert_ne!(bounded_part("a:b"), bounded_part("a:c"));
        assert_eq!(bounded_part("New York"), bounded_part("New York"));
    }

    #[tokio::test]
    async fn unreachable_redis_fails_open_quickly() {
        // Port 1 on loopback refuses connections; nothing leaves the machine.
        let cache = Cache::new("redis://127.0.0.1:1", "ib-test:").await.unwrap();
        let start = Instant::now();
        let key = cache.key("weather", &["paris"]);
        assert_eq!(cache.get::<String>(&key).await, None);
        cache.set(&key, &"value", 30).await;
        assert!(cache.ping().await.is_err());
        assert!(start.elapsed() < Duration::from_secs(3));
    }

    #[tokio::test]
    async fn disabled_cache_misses_skips_writes_and_never_connects() {
        let cache = Cache::disabled("ib-test:");
        assert!(!cache.is_enabled());
        let key = cache.key("weather", &["paris"]);
        assert_eq!(key, "ib-test:weather:paris");
        cache.set(&key, &"value", 30).await;
        assert_eq!(cache.get::<String>(&key).await, None);
        assert!(cache.ping().await.is_err());
        // No connection attempt was made, so there is no failure to back off from.
        let state = cache.state.lock().await;
        assert!(state.conn.is_none() && state.last_failure.is_none());
    }

    #[tokio::test]
    async fn invalid_url_is_a_startup_error() {
        let err = Cache::new("not a url", "x:")
            .await
            .err()
            .expect("invalid URL must fail")
            .to_string();
        assert!(err.contains("REDIS_URL"), "{err}");
    }
}
