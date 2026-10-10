use axum::http::HeaderValue;
use std::env;
use std::net::IpAddr;

/// Dev-server origins allowed when `CORS_ORIGINS` is unset: the Vite dev server.
const DEFAULT_CORS_ORIGINS: &str = "http://127.0.0.1:5173,http://localhost:5173";

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
            jwt_secret: required_env("JWT_SECRET")?,
            cors_origins: parse_cors_origins(env::var("CORS_ORIGINS").ok().as_deref())?,
            github_api_token: optional_env("GITHUB_API_TOKEN"),
            openweather_api_key: optional_env("OPENWEATHER_API_KEY"),
            newsapi_api_key: optional_env("NEWSAPI_API_KEY"),
        })
    }
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
    use super::{normalize_optional, parse_app_host, parse_cors_origins, require};
    use std::net::IpAddr;

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
}
