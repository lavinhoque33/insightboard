use std::env;
use std::net::IpAddr;

/// Application configuration loaded from environment variables
#[derive(Clone, Debug)]
pub struct Config {
    pub app_host: IpAddr,
    pub app_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub github_api_token: Option<String>,
    pub openweather_api_key: Option<String>,
    pub newsapi_api_key: Option<String>,
    pub coinmarketcap_api_key: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            app_host: parse_app_host(env::var("APP_HOST").ok().as_deref())?,
            app_port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL")
                .expect("REDIS_URL must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            github_api_token: optional_env("GITHUB_API_TOKEN"),
            openweather_api_key: optional_env("OPENWEATHER_API_KEY"),
            newsapi_api_key: optional_env("NEWSAPI_API_KEY"),
            coinmarketcap_api_key: optional_env("COINMARKETCAP_API_KEY"),
        })
    }
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
    use super::{normalize_optional, parse_app_host};
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
}
