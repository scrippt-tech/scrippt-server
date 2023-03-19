use redis::{AsyncCommands, Client, RedisError};

pub struct RedisRepository {
    pub client: Client,
}

impl RedisRepository {
    /// Create a new RedisRepository instance
    pub fn new(conn_string: &str) -> Self {
        let client = Client::open(conn_string).unwrap();
        Self { client }
    }

    /// Get a value from Redis
    pub async fn get(&self, key: &str) -> Result<String, RedisError> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let res: Option<String> = con.get(key).await?;
        Ok(res.unwrap_or_default())
    }

    /// Set a value in Redis
    pub async fn set(&self, key: &str, value: &str) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.set(key, value).await?;
        Ok(())
    }

    /// Delete a value from Redis
    pub async fn del(&self, key: &str) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.del(key).await?;
        Ok(())
    }

    /// Check if a key exists in Redis
    pub async fn exists(&self, key: &str) -> Result<bool, RedisError> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let res: bool = con.exists(key).await?;
        Ok(res)
    }

    /// Set a key to expire after a given number of seconds
    pub async fn expire(&self, key: &str, seconds: usize) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.expire(key, seconds).await?;
        Ok(())
    }
}
