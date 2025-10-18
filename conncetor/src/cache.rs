use redis::{AsyncTypedCommands, Client, RedisResult};

#[derive(Debug)]
pub struct CacheClient {
    remote: Client,
}

impl CacheClient {
    pub fn new() -> anyhow::Result<Self> {
        let redis_url = std::env::var("REDIS_URL")?;

        let client = Client::open(redis_url.as_str())?;

        Ok(Self { remote: client })
    }

    pub async fn ping_remote(&self) -> RedisResult<()> {
        let mut conn = self.remote.get_multiplexed_async_connection().await?;

        conn.ping().await?;

        Ok(())
    }
}
