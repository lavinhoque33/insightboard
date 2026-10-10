use axum::http::HeaderValue;
use reqwest::Url;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::time::Duration;

/// Dev-server origins allowed when `CORS_ORIGINS` is unset: the Vite dev server.
const DEFAULT_CORS_ORIGINS: &str = "http://127.0.0.1:5173,http://localhost:5173";

/// HS256 needs a key at least as long as its 256-bit output (M4 decision 8).
pub const MIN_JWT_SECRET_BYTES: usize = 32;

/// A `governor` quota: sustained requests per minute and burst size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RateLimit {
    pub per_minute: NonZeroU32,
    pub burst: NonZeroU32,
}

/// Provider base URLs; the defaults are the real APIs, tests point them at local mocks.
#[derive(Clone, Debug)]
pub struct ProviderUrls {
    pub github: Url,
    pub openweather: Url,
    pub newsapi: Url,
    pub coingecko: Url,
}

/// Application configuration loaded from environment variables
#[derive(Clone, Debug)]
pub struct Config {
    pub app_host: IpAddr,
    pub app_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<HeaderValue>,
    pub github_api_token: Option<String>,
    pub openweather_api_key: Option<String>,
    pub newsapi_api_key: Option<String>,
    pub provider_urls: ProviderUrls,
    pub provider_connect_timeout: Duration,
    pub provider_timeout: Duration,
    pub status_connect_timeout: Duration,
    pub status_timeout: Duration,
    /// Exact loopback `ip:port` fixtures the status widget may reach (development only).
    pub status_allowed_fixtures: Vec<SocketAddr>,
    /// Server-side limit for handling one request.
    pub request_timeout: Duration,
    pub auth_rate_limit: RateLimit,
    pub data_rate_limit: RateLimit,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            app_host: parse_app_host(env::var("APP_HOST").ok().as_deref())?,
            app_port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e| anyhow::anyhow!("APP_PORT must be a port number: {e}"))?,
            database_url: required_env("DATABASE_URL")?,
            redis_url: required_env("REDIS_URL")?,
            jwt_secret: check_jwt_secret(required_env("JWT_SECRET")?)?,
            cors_origins: parse_cors_origins(env::var("CORS_ORIGINS").ok().as_deref())?,
            github_api_token: optional_env("GITHUB_API_TOKEN"),
            openweather_api_key: optional_env("OPENWEATHER_API_KEY"),
            newsapi_api_key: optional_env("NEWSAPI_API_KEY"),
            provider_urls: ProviderUrls {
                github: base_url_env("GITHUB_API_BASE_URL", "https://api.github.com")?,
                openweather: base_url_env(
                    "OPENWEATHER_API_BASE_URL",
                    "https://api.openweathermap.org",
                )?,
                newsapi: base_url_env("NEWSAPI_API_BASE_URL", "https://newsapi.org")?,
                coingecko: base_url_env("COINGECKO_API_BASE_URL", "https://api.coingecko.com")?,
            },
            provider_connect_timeout: millis_env("PROVIDER_CONNECT_TIMEOUT_MS", 3_000)?,
            provider_timeout: millis_env("PROVIDER_TIMEOUT_MS", 8_000)?,
            status_connect_timeout: millis_env("STATUS_CONNECT_TIMEOUT_MS", 2_000)?,
            status_timeout: millis_env("STATUS_TIMEOUT_MS", 4_000)?,
            status_allowed_fixtures: parse_fixtures(
                env::var("STATUS_ALLOWED_FIXTURES").ok().as_deref(),
            )?,
            request_timeout: millis_env("REQUEST_TIMEOUT_MS", 10_000)?,
            auth_rate_limit: RateLimit {
                per_minute: nonzero_env("AUTH_RATE_LIMIT_PER_MINUTE", 10)?,
                burst: nonzero_env("AUTH_RATE_LIMIT_BURST", 5)?,
            },
            data_rate_limit: RateLimit {
                per_minute: nonzero_env("DATA_RATE_LIMIT_PER_MINUTE", 60)?,
                burst: nonzero_env("DATA_RATE_LIMIT_BURST", 30)?,
            },
        })
    }
}

fn check_jwt_secret(secret: String) -> anyhow::Result<String> {
    if secret.len() < MIN_JWT_SECRET_BYTES {
        anyhow::bail!(
            "JWT_SECRET must be at least {MIN_JWT_SECRET_BYTES} bytes (generate one with: openssl rand -hex 32)"
        );
    }
    Ok(secret)
}

/// An optional variable parsed with `parse`; unset or blank means `default`.
fn parse_env<T>(
    key: &str,
    default: T,
    parse: impl FnOnce(&str) -> Option<T>,
    expected: &str,
) -> anyhow::Result<T> {
    match optional_env(key) {
        None => Ok(default),
        Some(raw) => parse(&raw).ok_or_else(|| anyhow::anyhow!("{key} must be {expected}")),
    }
}

fn millis_env(key: &str, default_ms: u64) -> anyhow::Result<Duration> {
    parse_env(
        key,
        Duration::from_millis(default_ms),
        parse_millis,
        "a positive number of milliseconds",
    )
}

fn parse_millis(raw: &str) -> Option<Duration> {
    raw.parse::<u64>()
        .ok()
        .filter(|ms| *ms > 0)
        .map(Duration::from_millis)
}

fn nonzero_env(key: &str, default: u32) -> anyhow::Result<NonZeroU32> {
    let default = NonZeroU32::new(default).unwrap_or(NonZeroU32::MIN);
    parse_env(key, default, |raw| raw.parse().ok(), "a positive integer")
}

fn base_url_env(key: &str, default: &str) -> anyhow::Result<Url> {
    let raw = optional_env(key).unwrap_or_else(|| default.to_string());
    parse_base_url(&raw).ok_or_else(|| {
        anyhow::anyhow!("{key} must be an http(s) URL with a host and no query or fragment")
    })
}

fn parse_base_url(raw: &str) -> Option<Url> {
    let url = Url::parse(raw).ok()?;
    let ok = matches!(url.scheme(), "http" | "https")
        && url.host().is_some()
        && !url.cannot_be_a_base()
        && url.query().is_none()
        && url.fragment().is_none()
        && url.username().is_empty()
        && url.password().is_none();
    ok.then_some(url)
}

/// `STATUS_ALLOWED_FIXTURES`: comma-separated exact loopback `ip:port` entries; empty by default.
fn parse_fixtures(value: Option<&str>) -> anyhow::Result<Vec<SocketAddr>> {
    let mut fixtures = Vec::new();
    for entry in value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|e| !e.is_empty())
    {
        let addr: SocketAddr = entry.parse().map_err(|_| {
            anyhow::anyhow!("STATUS_ALLOWED_FIXTURES entry {entry:?} must be ip:port")
        })?;
        if !addr.ip().is_loopback() {
            anyhow::bail!("STATUS_ALLOWED_FIXTURES entry {entry:?} must be a loopback address");
        }
        fixtures.push(addr);
    }
    Ok(fixtures)
}

fn required_env(key: &str) -> anyhow::Result<String> {
    require(key, env::var(key).ok())
}

/// A required variable must be set and non-blank; the error names it.
fn require(key: &str, value: Option<String>) -> anyhow::Result<String> {
    normalize_optional(value)
        .ok_or_else(|| anyhow::anyhow!("{key} must be set to a non-empty value (see .env.example)"))
}

/// Parse `CORS_ORIGINS`: comma-separated exact origins (`scheme://host[:port]`).
/// Unset or blank means the Vite dev origins.
fn parse_cors_origins(value: Option<&str>) -> anyhow::Result<Vec<HeaderValue>> {
    let raw = match value.map(str::trim) {
        None | Some("") => DEFAULT_CORS_ORIGINS,
        Some(raw) => raw,
    };
    let mut origins = Vec::new();
    for entry in raw.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        origins.push(parse_origin(entry)?);
    }
    if origins.is_empty() {
        anyhow::bail!("CORS_ORIGINS must list at least one origin");
    }
    Ok(origins)
}

fn parse_origin(entry: &str) -> anyhow::Result<HeaderValue> {
    let invalid =
        |why: &str| anyhow::anyhow!("CORS_ORIGINS has an invalid origin {entry:?}: {why}");
    let authority = entry
        .strip_prefix("http://")
        .or_else(|| entry.strip_prefix("https://"))
        .ok_or_else(|| invalid("must start with http:// or https://"))?;
    if authority.is_empty()
        || authority
            .contains(|c: char| matches!(c, '/' | '?' | '#' | '@' | '*') || c.is_whitespace())
    {
        return Err(invalid(
            "expected scheme://host[:port] with no path, wildcard, or credentials",
        ));
    }
    HeaderValue::from_str(entry).map_err(|_| invalid("not a valid header value"))
}

fn optional_env(key: &str) -> Option<String> {
    normalize_optional(env::var(key).ok())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

/// Parse the bind host; unset means loopback.
fn parse_app_host(value: Option<&str>) -> anyhow::Result<IpAddr> {
    match value {
        None => Ok(IpAddr::from([127, 0, 0, 1])),
        Some(raw) => raw
            .trim()
            .parse()
            .map_err(|e| anyhow::anyhow!("APP_HOST must be an IP address: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        check_jwt_secret, normalize_optional, parse_app_host, parse_base_url, parse_cors_origins,
        parse_fixtures, parse_millis, require,
    };
    use std::net::IpAddr;
    use std::time::Duration;

    #[test]
    fn app_host_defaults_to_loopback() {
        assert_eq!(parse_app_host(None).unwrap(), IpAddr::from([127, 0, 0, 1]));
    }

    #[test]
    fn app_host_parses_ip_and_rejects_garbage() {
        assert_eq!(
            parse_app_host(Some(" ::1 ")).unwrap(),
            IpAddr::from([0u16, 0, 0, 0, 0, 0, 0, 1])
        );
        let err = parse_app_host(Some("localhost")).unwrap_err().to_string();
        assert!(err.contains("APP_HOST"));
    }

    #[test]
    fn normalize_optional_strips_blank_values() {
        assert_eq!(normalize_optional(None), None);
        assert_eq!(normalize_optional(Some(String::new())), None);
        assert_eq!(normalize_optional(Some("   ".to_string())), None);
    }

    #[test]
    fn normalize_optional_trims_and_keeps_non_empty() {
        assert_eq!(
            normalize_optional(Some("  my-key  ".to_string())),
            Some("my-key".to_string())
        );
    }

    fn origins(value: Option<&str>) -> Vec<String> {
        parse_cors_origins(value)
            .unwrap()
            .iter()
            .map(|o| o.to_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn cors_origins_default_to_vite_dev_origin() {
        let expected = ["http://127.0.0.1:5173", "http://localhost:5173"];
        assert_eq!(origins(None), expected);
        assert_eq!(origins(Some("  ")), expected);
    }

    #[test]
    fn cors_origins_parse_a_trimmed_list() {
        assert_eq!(
            origins(Some(" https://app.example.com , http://localhost:3000,")),
            ["https://app.example.com", "http://localhost:3000"]
        );
    }

    #[test]
    fn cors_origins_reject_invalid_entries() {
        for bad in [
            "*",
            "localhost:5173",
            "ftp://example.com",
            "https://example.com/",
            "https://example.com/path",
            "https://user@example.com",
            "https://*.example.com",
            "http://",
            ",",
        ] {
            let err = parse_cors_origins(Some(bad)).unwrap_err().to_string();
            assert!(err.contains("CORS_ORIGINS"), "{bad}: {err}");
        }
    }

    #[test]
    fn required_variables_name_themselves_when_missing_or_blank() {
        for value in [None, Some(String::new()), Some("  ".to_string())] {
            let err = require("JWT_SECRET", value).unwrap_err().to_string();
            assert!(err.contains("JWT_SECRET"), "{err}");
        }
        assert_eq!(require("JWT_SECRET", Some(" x ".into())).unwrap(), "x");
    }

    #[test]
    fn short_jwt_secret_is_rejected_by_name() {
        let err = check_jwt_secret("x".repeat(31)).unwrap_err().to_string();
        assert!(err.contains("JWT_SECRET") && err.contains("32"), "{err}");
        assert!(check_jwt_secret("x".repeat(32)).is_ok());
    }

    #[test]
    fn base_urls_must_be_plain_http_urls() {
        assert!(parse_base_url("https://api.github.com").is_some());
        assert!(parse_base_url("http://127.0.0.1:9999/prefix").is_some());
        for bad in [
            "api.github.com",
            "ftp://x.example",
            "https://x.example/?a=1",
            "https://u:p@x.example",
            "mailto:a@b.c",
        ] {
            assert!(parse_base_url(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn fixtures_accept_only_exact_loopback_socket_addresses() {
        assert!(parse_fixtures(None).unwrap().is_empty());
        assert!(parse_fixtures(Some(" ")).unwrap().is_empty());
        let fixtures = parse_fixtures(Some("127.0.0.1:18080, [::1]:18081")).unwrap();
        assert_eq!(fixtures.len(), 2);
        for bad in ["127.0.0.1", "10.0.0.1:80", "localhost:18080", "0.0.0.0:80"] {
            let err = parse_fixtures(Some(bad)).unwrap_err().to_string();
            assert!(err.contains("STATUS_ALLOWED_FIXTURES"), "{bad}: {err}");
        }
    }

    #[test]
    fn millis_must_be_positive() {
        assert_eq!(parse_millis("250"), Some(Duration::from_millis(250)));
        assert_eq!(parse_millis("0"), None);
        assert_eq!(parse_millis("-1"), None);
    }
}
