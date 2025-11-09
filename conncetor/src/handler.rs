use std::net::SocketAddrV4;
use std::time::Duration;

use axum::Router;
use axum::routing;
use tokio::net::TcpListener;
use tracing::debug;

mod connect;
mod health;
mod result;

use crate::cache::CacheClient;
use crate::config::AppConfig;
use crate::registry::{ConsulRegistry, HealthCheck, RegisterService};
use crate::state::AppState;
use crate::upstream::UpstreamRouter;

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

async fn init_registry() -> anyhow::Result<ConsulRegistry> {
    let registry = ConsulRegistry::new()?;

    debug!("Registry center client initialized");

    Ok(registry)
}

async fn init_connector(app_config: &AppConfig, registry: &ConsulRegistry) -> anyhow::Result<()> {
    let registry_service = RegisterService {
        id: app_config.service_id.to_string(),
        name: format!("connector-service-{}", app_config.service_id),
        address: app_config.server_host.clone(),
        port: app_config.server_port,
        health: HealthCheck {
            http: format!(
                "http://{}:{}/health/check",
                app_config.server_host, app_config.server_port
            ),
            interval: "10s".to_string(),
        },
    };

    registry
        .register_service(&registry_service)
        .await
        .map_err(|err| anyhow::anyhow!("Error when registering service: {}", err))?;

    debug!(
        "Connector initialized with config: {:?}",
        format!("{}:{}", app_config.server_host, app_config.server_port)
    );

    Ok(())
}

async fn init_upstreams(registry: ConsulRegistry) -> anyhow::Result<UpstreamRouter> {
    let mut upstreams = UpstreamRouter::new(registry, None);

    upstreams.full_update().await.map_err(|err| {
        anyhow::anyhow!(
            "Error when fully updating initial upstream service lists: {}",
            err
        )
    })?;

    // If succeeded to update, spawn a background task to periodically update the lists.
    let mut cloned_upstreams = upstreams.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            match cloned_upstreams.full_update().await {
                Ok(_) => {
                    debug!("Successfully updated upstream service lists");
                }
                Err(err) => {
                    debug!(
                        "Error when periodically updating upstream service lists: {}",
                        err
                    );
                }
            }
        }
    });

    Ok(upstreams)
}

async fn new_router(app_config: &AppConfig) -> anyhow::Result<Router> {
    let cache = init_cache().await?;
    let registry = init_registry().await?;
    let upstreams = init_upstreams(registry.clone()).await?;

    init_connector(app_config, &registry).await?;

    let app_state = AppState::new(app_config.clone(), cache, registry, upstreams);

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
