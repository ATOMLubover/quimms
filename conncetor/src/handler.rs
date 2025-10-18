use std::net::SocketAddrV4;

use axum::Router;
use axum::routing::{self};
use tokio::net::TcpListener;
use tracing::debug;

mod connect;
mod health;

use crate::cache::CacheClient;
use crate::config::AppConfig;
use crate::registry::RegistryClient;
use crate::state::AppState;

async fn init_cache() -> anyhow::Result<CacheClient> {
    let cache = CacheClient::new()
        .map_err(|err| anyhow::anyhow!("Error when initializing cache client: {}", err))?;

    // Test the Redis with an initial PING command to ensure connectivity
    cache
        .ping_remote()
        .await
        .map_err(|err| anyhow::anyhow!("Error when PING Redis: {}", err))?;

    debug!("Redis connected");

    Ok(cache)
}

async fn init_registry() -> anyhow::Result<RegistryClient> {
    let registry = RegistryClient::new()?;

    debug!("Registry center client initialized");

    Ok(registry)
}

async fn new_router(app_config: &AppConfig) -> anyhow::Result<Router> {
    let cache = init_cache().await?;
    let registry = init_registry().await?;

    let app_state = AppState::new(app_config.clone(), cache, registry);

    let health_router = Router::new().route("/check", routing::get(health::health_check));

    let router: Router = Router::new()
        .nest("/health", health_router)
        .with_state(app_state);

    Ok(router)
}

async fn bind_addr(host: &str, port: u16) -> anyhow::Result<TcpListener> {
    let addr: SocketAddrV4 = format!("{}:{}", host, port)
        .parse()
        .map_err(|err| anyhow::anyhow!("Error when parsing listening address: {}", err))?;

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|err| anyhow::anyhow!("Error when binding to address {}: {}", addr, err))?;

    debug!("Server now listening on {}", addr);

    Ok(listener)
}

async fn signal_term() {
    debug!("SIGNAL TERM receiver installed");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL-C signal handler");

    debug!("SIGNAL TERM received, shutting down gracefully...");
}

pub async fn run_server(app_config: &AppConfig) -> anyhow::Result<()> {
    let router = new_router(app_config).await?;

    let listener = bind_addr(&app_config.server_host, app_config.server_port).await?;

    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(signal_term())
        .await
        .map_err(|err| anyhow::anyhow!("Error running server: {}", err))?;

    Ok(())
}
