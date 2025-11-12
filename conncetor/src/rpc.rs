use std::time::Duration;

use anyhow::anyhow;
use tonic::transport::Channel;

mod dispatch;

use crate::{
    registry::{
        ConsulRegistry,
        model::{HeathCheck, Registry, ServiceEntry},
        store::ConsistHashStore,
    },
    rpc::dispatch::{DispatchServer, dispatcher::dispatch_service_server::DispatchServiceServer},
    state::AppState,
};

async fn transformer(entry: ServiceEntry) -> Channel {
    let addr = format!("http://{}", entry.info().address());

    Channel::from_shared(addr).unwrap().connect().await.unwrap()
}

const REPLICAS: usize = 5;

const DEFAULT_HASHER: fn(&str) -> u64 = |key: &str| {
    use std::hash::Hasher as StdHasher;

    let mut default_hasher = twox_hash::XxHash64::with_seed(0);

    default_hasher.write(key.as_bytes());

    default_hasher.finish()
};

const USER_SERVICE_PREFIX: &str = "user-service";

pub async fn init_user_service(consul_addr: &str) -> anyhow::Result<ConsulRegistry<Channel>> {
    let store = ConsistHashStore::new(REPLICAS, DEFAULT_HASHER);

    let registry = ConsulRegistry::new(consul_addr, USER_SERVICE_PREFIX, store).map_err(|err| {
        anyhow!(
            "Error when initiating user service registry client: {}",
            err
        )
    })?;

    registry
        .update_store(transformer)
        .await
        .map_err(|err| anyhow!("Error when updating user service registry store: {}", err))?;

    registry.spawn_update_store(transformer);

    Ok(registry)
}

const CHANNEL_SERVICE_PREFIX: &str = "channel-service";

pub async fn init_channel_service(consul_addr: &str) -> anyhow::Result<ConsulRegistry<Channel>> {
    let store = ConsistHashStore::new(REPLICAS, DEFAULT_HASHER);

    let registry =
        ConsulRegistry::new(consul_addr, CHANNEL_SERVICE_PREFIX, store).map_err(|err| {
            anyhow!(
                "Error when initiating channel service registry client: {}",
                err
            )
        })?;

    registry.update_store(transformer).await.map_err(|err| {
        anyhow!(
            "Error when updating channel service registry store: {}",
            err
        )
    })?;

    registry.spawn_update_store(transformer);

    Ok(registry)
}

const MESSAGE_SERVICE_PREFIX: &str = "message-service";

pub async fn init_message_service(consul_addr: &str) -> anyhow::Result<ConsulRegistry<Channel>> {
    let store = ConsistHashStore::new(REPLICAS, DEFAULT_HASHER);

    let registry =
        ConsulRegistry::new(consul_addr, MESSAGE_SERVICE_PREFIX, store).map_err(|err| {
            anyhow!(
                "Error when initiating message service registry client: {}",
                err
            )
        })?;

    registry.update_store(transformer).await.map_err(|err| {
        anyhow!(
            "Error when updating message service registry store: {}",
            err
        )
    })?;

    registry.spawn_update_store(transformer);

    Ok(registry)
}

pub async fn run_dispatch_server(state: &AppState) -> anyhow::Result<()> {
    let dispatch_addr = format!(
        "{}:{}",
        state.config().grpc_host(),
        state.config().grpc_port()
    );

    const DISPATCH_SERVICE_PREFIX: &str = "dispatch-service";

    let registry = ConsulRegistry::new(
        &format!(
            "{}:{}",
            state.config().consul_host(),
            state.config().consul_port()
        ),
        DISPATCH_SERVICE_PREFIX,
        ConsistHashStore::<()>::new(REPLICAS, DEFAULT_HASHER),
    )
    .map_err(|err| {
        anyhow!(
            "Error when initiating dispatch service registry client: {}",
            err
        )
    })?;

    const TTL_SECONDS: i32 = 60;

    let ttl = Duration::from_secs(TTL_SECONDS as u64);

    let check_id = format!(
        "{}-{}",
        DISPATCH_SERVICE_PREFIX,
        state.config().service_id()
    );

    // A background task to refresh service registry is spwawned automatically.
    registry
        .register(
            Registry::new(
                state.config().service_id().to_string(),
                DISPATCH_SERVICE_PREFIX.to_string(),
                state.config().grpc_host().to_string(),
                state.config().grpc_port(),
                HeathCheck::new(ttl.clone(), check_id, DISPATCH_SERVICE_PREFIX.to_string()),
            ),
            ttl.clone(),
        )
        .await?;

    let dispatch_server = DispatchServer::new(&state);

    tonic::transport::Server::builder()
        .add_service(DispatchServiceServer::new(dispatch_server))
        .serve(dispatch_addr.parse()?)
        .await
        .map_err(|err| anyhow!("Error running dispatch gRPC server: {}", err))
}
