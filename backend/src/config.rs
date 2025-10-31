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
            github_api_token: env::var("GITHUB_API_TOKEN").ok(),
            openweather_api_key: env::var("OPENWEATHER_API_KEY").ok(),
            newsapi_api_key: env::var("NEWSAPI_API_KEY").ok(),
            coinmarketcap_api_key: env::var("COINMARKETCAP_API_KEY").ok(),
        })
    }
}
