use tracing::debug;
use tracing_subscriber::EnvFilter;

mod cache;
mod config;
mod handler;
mod registry;
mod service;
mod state;
mod transfer;

use crate::config::AppConfig;

async fn init_env() -> anyhow::Result<()> {
    dotenvy::dotenv().map_err(|err| anyhow::anyhow!("Error when loading env: {}", err))?;

    debug!("Environment variables loaded from .env file");

    Ok(())
}

async fn init_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    debug!("Logger initialized");
}

async fn init_config() -> anyhow::Result<AppConfig> {
    let config = AppConfig::try_from_file(None)
        .map_err(|err| anyhow::anyhow!("Error when loading config: {}", err))?;

    debug!("Configuration loaded: {:?}", config);

    Ok(config)
}

pub async fn run() -> anyhow::Result<()> {
    init_env().await?;

    init_logger().await;

    let config = init_config().await?;

    handler::run_server(&config).await?;

    Ok(())
}
