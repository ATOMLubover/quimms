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
}
