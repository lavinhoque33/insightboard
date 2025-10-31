use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};

/// Redis cache wrapper
#[derive(Clone)]
pub struct Cache {
    client: Client,
}

impl Cache {
    /// Create a new Redis cache connection
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = Client::open(redis_url)?;
        
        // Test the connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        
        Ok(Self { client })
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> anyhow::Result<MultiplexedConnection> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }

    /// Get a value from cache
    pub async fn get<T>(&self, key: &str) -> anyhow::Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let mut conn = self.get_connection().await?;
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with TTL (in seconds)
    pub async fn set<T>(&self, key: &str, value: &T, ttl: usize) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)?;
        conn.set_ex(key, serialized, ttl as u64).await?;
        Ok(())
    }

    /// Delete a value from cache
    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        conn.del(key).await?;
        Ok(())
    }

    /// Check if a key exists in cache
    pub async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }
}
