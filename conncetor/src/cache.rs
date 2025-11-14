use anyhow::anyhow;
use redis::{AsyncTypedCommands, Client, RedisResult};

#[derive(Debug)]
pub struct CacheClient {
    remote: Client,
}

impl CacheClient {
    pub async fn new() -> anyhow::Result<Self> {
        let redis_url = std::env::var("REDIS_URL")?;

        let client = Client::open(redis_url.as_str())?;

        client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| anyhow!("Error when connecting to Redis server: {}", err))?
            .ping()
            .await
            .map_err(|err| anyhow!("Error when pinging Redis server: {}", err))?;

        Ok(Self { remote: client })
    }

    pub async fn ping_remote(&self) -> RedisResult<()> {
        let mut conn = self.remote.get_multiplexed_async_connection().await?;

        conn.ping().await?;

        Ok(())
    }

    pub async fn set_ex_nx(
        &self,
        key: &str,
        value: &str,
        ttl_sec: Option<i64>,
    ) -> RedisResult<bool> {
        let conn = &mut self.remote.get_multiplexed_async_connection().await?;

        // If ttl is Some, pass it as ARGV[2], otherwise pass an empty string
        let result = match ttl_sec {
            Some(ttl) => {
                let mut pipeline = redis::pipe();

                pipeline
                    .set_nx(key, value)
                    .expire(key, ttl)
                    .query_async(conn)
                    .await?
            }
            None => conn.set_nx(key, value).await?,
        };

        Ok(result)
    }

    pub async fn hash_set(&self, hash_key: &str, field: &str, value: &str) -> RedisResult<bool> {
        let conn = &mut self.remote.get_multiplexed_async_connection().await?;

        let result = conn.hset(hash_key, field, value).await?;

        match result {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(redis::RedisError::from((
                redis::ErrorKind::ResponseError,
                "Unexpected response from Redis HSET",
            ))),
        }
    }

    pub async fn hash_delete(&self, hash_key: &str, field: &str) -> RedisResult<bool> {
        let conn = &mut self.remote.get_multiplexed_async_connection().await?;

        let result = conn.hdel(hash_key, field).await?;

        match result {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(redis::RedisError::from((
                redis::ErrorKind::ResponseError,
                "Unexpected response from Redis HDEL",
            ))),
        }
    }
}
