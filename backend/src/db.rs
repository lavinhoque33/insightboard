use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// How long a request waits for a pooled connection before failing (instead of hanging).
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(3);

/// Database connection wrapper
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(ACQUIRE_TIMEOUT)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Wrap an existing pool (tests).
    #[cfg(test)]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Readiness probe: one round trip.
    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}
