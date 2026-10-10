use axum::{extract::State, Json};
use reqwest::{redirect, Client, Url};
use serde::{Deserialize, Serialize};
use std::{
    error::Error as _,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Semaphore, task::JoinSet};

use super::{
    http::USER_AGENT,
    net_policy::{is_blocked_ip, BlockedAddress, DnsFailure, PolicyResolver},
};
use crate::{
    auth::UserCtx,
    error::{AppError, Result},
    extract::ApiQuery,
    AppState,
};

/// Most URLs one request may check (422 beyond).
pub const MAX_STATUS_URLS: usize = 10;
/// URLs checked at the same time.
pub const STATUS_CONCURRENCY: usize = 5;
/// Longest accepted URL, in bytes.
const MAX_URL_BYTES: usize = 2048;
/// Results are cached per user for 2 minutes.
const STATUS_CACHE_TTL_SECS: u64 = 120;

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    /// Comma-separated URLs.
    pub urls: Option<String>,
}

/// Generic failure classes; library error text is never returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusErrorClass {
    InvalidUrl,
    BlockedAddress,
    DnsError,
    Timeout,
    ConnectError,
    RequestError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCheck {
    pub url: String,
    pub status: u16,
    pub response_time_ms: u64,
    pub error: Option<StatusErrorClass>,
}

impl StatusCheck {
    fn failed(url: String, class: StatusErrorClass) -> Self {
        Self {
            url,
            status: 0,
            response_time_ms: 0,
            error: Some(class),
        }
    }
}

/// The status client held in `AppState`: no redirects (a 3xx is the result), no proxy (the
/// dialled address must be the vetted one), and DNS through [`PolicyResolver`].
pub fn status_client(connect_timeout: Duration, timeout: Duration) -> anyhow::Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(connect_timeout)
        .timeout(timeout)
        .redirect(redirect::Policy::none())
        .no_proxy()
        .dns_resolver(Arc::new(PolicyResolver))
        .build()
        .map_err(|e| anyhow::anyhow!("failed to build the status HTTP client: {e}"))
}

/// Check a user-supplied URL before any connection: http(s) only, no credentials, bounded
/// length, and an IP-literal host must be allowed (or an exact fixture). Domain hosts are
/// checked by the resolver at connect time.
pub fn vet_url(raw: &str, fixtures: &[SocketAddr]) -> std::result::Result<Url, StatusErrorClass> {
    use StatusErrorClass::{BlockedAddress, InvalidUrl};
    if raw.len() > MAX_URL_BYTES {
        return Err(InvalidUrl);
    }
    let url = Url::parse(raw).map_err(|_| InvalidUrl)?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(InvalidUrl);
    }
    let host = url.host_str().ok_or(InvalidUrl)?;
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = bare.parse::<IpAddr>() {
        let port = url.port_or_known_default().ok_or(InvalidUrl)?;
        if fixtures.contains(&SocketAddr::new(ip, port)) {
            return Ok(url);
        }
        if is_blocked_ip(ip) {
            return Err(BlockedAddress);
        }
    }
    Ok(url)
}

fn classify(error: &reqwest::Error) -> StatusErrorClass {
    let mut source = error.source();
    while let Some(cause) = source {
        if cause.is::<BlockedAddress>() {
            return StatusErrorClass::BlockedAddress;
        }
        if cause.is::<DnsFailure>() {
            return StatusErrorClass::DnsError;
        }
        source = cause.source();
    }
    if error.is_timeout() {
        StatusErrorClass::Timeout
    } else if error.is_connect() {
        StatusErrorClass::ConnectError
    } else {
        StatusErrorClass::RequestError
    }
}

async fn check_one(client: &Client, raw: String, url: Url) -> StatusCheck {
    let start = Instant::now();
    match client.get(url).send().await {
        Ok(response) => StatusCheck {
            url: raw,
            status: response.status().as_u16(),
            response_time_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
            error: None,
        },
        Err(e) => {
            let class = classify(&e);
            tracing::debug!(?class, "status check failed");
            StatusCheck::failed(raw, class)
        }
    }
}

/// Check every URL, [`STATUS_CONCURRENCY`] at a time, returning results in request order. The
/// tasks are owned by the `JoinSet` and abort if the request is dropped.
pub async fn check_all(
    client: &Client,
    fixtures: &[SocketAddr],
    urls: &[String],
) -> Vec<StatusCheck> {
    let permits = Arc::new(Semaphore::new(STATUS_CONCURRENCY));
    let mut tasks = JoinSet::new();
    for (index, raw) in urls.iter().enumerate() {
        let vetted = vet_url(raw, fixtures);
        let client = client.clone();
        let permits = Arc::clone(&permits);
        let raw = raw.clone();
        tasks.spawn(async move {
            let check = match vetted {
                Err(class) => StatusCheck::failed(raw, class),
                Ok(url) => {
                    let _permit = permits.acquire_owned().await;
                    check_one(&client, raw, url).await
                }
            };
            (index, check)
        });
    }
    let mut results: Vec<Option<StatusCheck>> = urls.iter().map(|_| None).collect();
    while let Some(joined) = tasks.join_next().await {
        match joined {
            Ok((index, check)) => {
                if let Some(slot) = results.get_mut(index) {
                    *slot = Some(check);
                }
            }
            Err(e) => tracing::error!(error = %e, "status check task failed"),
        }
    }
    results
        .into_iter()
        .zip(urls)
        .map(|(check, raw)| {
            check
                .unwrap_or_else(|| StatusCheck::failed(raw.clone(), StatusErrorClass::RequestError))
        })
        .collect()
}

pub async fn fetch_status_data(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<StatusQuery>,
) -> Result<Json<Vec<StatusCheck>>> {
    let urls: Vec<String> = query
        .urls
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .map(String::from)
        .collect();
    if urls.is_empty() {
        return Err(AppError::invalid("urls", "is required"));
    }
    if urls.len() > MAX_STATUS_URLS {
        return Err(AppError::invalid(
            "urls",
            format!("must list at most {MAX_STATUS_URLS} URLs"),
        ));
    }

    // Per user: one user's results never answer another's request.
    let user_id = user_ctx.user_id.to_string();
    let cache_key = state.cache.key("status", &[&user_id, &urls.join("\n")]);
    if let Some(cached) = state.cache.get::<Vec<StatusCheck>>(&cache_key).await {
        return Ok(Json(cached));
    }

    let checks = check_all(
        &state.http.status,
        &state.config.status_allowed_fixtures,
        &urls,
    )
    .await;
    state
        .cache
        .set(&cache_key, &checks, STATUS_CACHE_TTL_SECS)
        .await;
    Ok(Json(checks))
}

#[cfg(test)]
mod tests {
    use super::{vet_url, StatusErrorClass};
    use std::net::SocketAddr;

    #[test]
    fn only_plain_http_urls_pass() {
        for bad in [
            "ftp://example.com/",
            "file:///etc/passwd",
            "gopher://example.com",
            "javascript:alert(1)",
            "not a url",
            "http://user:pass@example.com/",
            "http://user@example.com/",
            "http://",
        ] {
            assert_eq!(
                vet_url(bad, &[]),
                Err(StatusErrorClass::InvalidUrl),
                "{bad}"
            );
        }
        let long = format!("https://example.com/{}", "a".repeat(3000));
        assert_eq!(vet_url(&long, &[]), Err(StatusErrorClass::InvalidUrl));
        assert!(vet_url("https://example.com/health", &[]).is_ok());
    }

    #[test]
    fn ip_literal_hosts_in_blocked_ranges_are_rejected() {
        for bad in [
            "http://127.0.0.1/",
            "http://127.1/",
            "http://0x7f000001/",
            "http://2130706433/",
            "http://0.0.0.0:8080/",
            "http://169.254.169.254/latest/meta-data/",
            "http://10.1.2.3/",
            "http://192.168.0.1/",
            "http://172.16.5.4/",
            "http://100.64.0.1/",
            "http://[::1]/",
            "http://[::ffff:127.0.0.1]/",
            "http://[fd00:ec2::254]/",
            "http://[fe80::1]/",
            "http://224.0.0.1/",
            "http://255.255.255.255/",
            "http://192.0.2.1/",
        ] {
            assert_eq!(
                vet_url(bad, &[]),
                Err(StatusErrorClass::BlockedAddress),
                "{bad}"
            );
        }
    }

    #[test]
    fn fixture_allowlist_admits_only_the_exact_socket_address() {
        let fixtures: Vec<SocketAddr> = vec!["127.0.0.1:18080".parse().unwrap()];
        assert!(vet_url("http://127.0.0.1:18080/ok", &fixtures).is_ok());
        assert_eq!(
            vet_url("http://127.0.0.1:18081/", &fixtures),
            Err(StatusErrorClass::BlockedAddress)
        );
        assert_eq!(
            vet_url("http://127.0.0.2:18080/", &fixtures),
            Err(StatusErrorClass::BlockedAddress)
        );
        assert_eq!(
            vet_url("http://127.0.0.1/", &fixtures),
            Err(StatusErrorClass::BlockedAddress)
        );
    }
}
