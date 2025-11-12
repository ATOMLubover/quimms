use anyhow::anyhow;
use tonic::transport::Channel;
use tracing::debug;
use tracing_subscriber::EnvFilter;

mod cache;
mod config;
mod consist_hash;
mod http;
mod message;
mod model;
mod registry;
mod rpc;
mod service;
mod state;
mod transfer;

use crate::{
    cache::CacheClient, config::AppConfig, registry::ConsulRegistry, rpc::run_dispatch_server,
    state::AppState,
};

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

async fn init_grpc_clients(
    config: &AppConfig,
) -> anyhow::Result<(
    ConsulRegistry<Channel>,
    ConsulRegistry<Channel>,
    ConsulRegistry<Channel>,
)> {
    let consul_addr = format!("{}:{}", config.consul_host(), config.consul_port());

    let user_resgitry = rpc::init_user_service(&consul_addr)
        .await
        .map_err(|err| anyhow!("Error when conecting to Consul user service: {}", err))?;
    let channel_registry = rpc::init_channel_service(&consul_addr)
        .await
        .map_err(|err| anyhow!("Error when conecting to Consul channel service: {}", err))?;
    let message_registry = rpc::init_message_service(&consul_addr)
        .await
        .map_err(|err| anyhow!("Error when conecting to Consul message service: {}", err))?;

    debug!("gRPC clients initialized");

    Ok((user_resgitry, channel_registry, message_registry))
}

async fn init_cache_client() -> anyhow::Result<CacheClient> {
    let cache_client = CacheClient::new()
        .await
        .map_err(|err| anyhow::anyhow!("Error when connecting to cache: {}", err))?;

    debug!("Cache client initialized");

    Ok(cache_client)
}

fn init_app_state(
    config: AppConfig,
    cache: CacheClient,
    user_registry: ConsulRegistry<Channel>,
    channel_registry: ConsulRegistry<Channel>,
    message_registry: ConsulRegistry<Channel>,
) -> AppState {
    let state = AppState::new(
        config,
        cache,
        user_registry,
        channel_registry,
        message_registry,
    );

    debug!("AppState initialized");

    state
}

async fn run_http_server(state: &AppState) -> anyhow::Result<()> {
    debug!("HTTP server running");

    http::run_server(state).await?;

    Ok(())
}

async fn run_grpc_server(state: &AppState) -> anyhow::Result<()> {
    debug!("gRPC server running");

    rpc::run_dispatch_server(state).await?;

    Ok(())
}

pub async fn run() -> anyhow::Result<()> {
    init_env().await?;

    init_logger().await;

    let config = init_config().await?;

    let cache = init_cache_client().await?;

    let (user_registry, channel_registry, message_registry) = init_grpc_clients(&config).await?;

    let app_state = init_app_state(
        config.clone(),
        cache,
        user_registry,
        channel_registry,
        message_registry,
    );

    let http_task = run_http_server(&app_state);

    let grpc_task = run_grpc_server(&app_state);

    tokio::try_join!(http_task, grpc_task)?;

    Ok(())
}
