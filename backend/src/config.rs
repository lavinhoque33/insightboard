use std::env;

/// Application configuration loaded from environment variables
#[derive(Clone, Debug)]
pub struct Config {
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

#[cfg(test)]
mod tests {
    use super::normalize_optional;

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
