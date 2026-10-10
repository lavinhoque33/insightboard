//! The shared outbound HTTP client for provider widgets (D5, D30, M4 decision 5).

use reqwest::{Client, RequestBuilder, Url};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::error::AppError;

/// Fixed `User-Agent` for every outbound request (providers reject requests without one, D30).
pub const USER_AGENT: &str = concat!(
    "InsightBoard/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/lavinhoque33/insightboard)"
);

/// Upper bound on a provider response body; larger bodies are an upstream error.
const MAX_PROVIDER_BODY_BYTES: usize = 4 * 1024 * 1024;

/// Build the one provider client held in `AppState`.
pub fn provider_client(connect_timeout: Duration, timeout: Duration) -> anyhow::Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(connect_timeout)
        .timeout(timeout)
        .build()
        .map_err(|e| anyhow::anyhow!("failed to build the provider HTTP client: {e}"))
}

/// `base` with `segments` appended as encoded path segments (never string interpolation).
pub fn endpoint(base: &Url, segments: &[&str]) -> Result<Url, AppError> {
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|_| AppError::Internal("provider base URL cannot be a base".to_string()))?
        .pop_if_empty()
        .extend(segments);
    Ok(url)
}

/// Send a provider request and decode a successful JSON body. Every failure is a generic 502;
/// the logged cause names the provider and never contains the URL (it may carry a key) or the
/// body.
pub async fn fetch_json<T: DeserializeOwned>(
    provider: &'static str,
    request: RequestBuilder,
) -> Result<T, AppError> {
    let mut response = request.send().await.map_err(|e| upstream(provider, e))?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::ExternalApi(format!(
            "{provider} returned status {status}"
        )));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| upstream(provider, e))? {
        if body.len() + chunk.len() > MAX_PROVIDER_BODY_BYTES {
            return Err(AppError::ExternalApi(format!(
                "{provider} response exceeded {MAX_PROVIDER_BODY_BYTES} bytes"
            )));
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|e| {
        // serde_json errors carry a position, not the body text.
        AppError::ExternalApi(format!("{provider} response did not decode: {e}"))
    })
}

fn upstream(provider: &'static str, error: reqwest::Error) -> AppError {
    let kind = if error.is_timeout() {
        "timed out"
    } else if error.is_connect() {
        "connection failed"
    } else {
        "request failed"
    };
    AppError::ExternalApi(format!("{provider} {kind}: {}", error.without_url()))
}

#[cfg(test)]
mod tests {
    use super::endpoint;
    use reqwest::Url;

    #[test]
    fn segments_are_percent_encoded() {
        let base = Url::parse("http://127.0.0.1:9/").unwrap();
        let url = endpoint(&base, &["users", "a/../b?x#y", "events"]).unwrap();
        assert_eq!(
            url.as_str(),
            "http://127.0.0.1:9/users/a%2F..%2Fb%3Fx%23y/events"
        );
        let prefixed = Url::parse("http://127.0.0.1:9/api").unwrap();
        assert_eq!(
            endpoint(&prefixed, &["v3"]).unwrap().as_str(),
            "http://127.0.0.1:9/api/v3"
        );
    }
}
