use anyhow::anyhow;
use tonic::transport::Channel;

use crate::registry::{ConsulRegistry, model::ServiceEntry, store::ConsistHashStore};

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
