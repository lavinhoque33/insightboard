use sqlx::{postgres::PgPoolOptions, PgPool};

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
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }
}
