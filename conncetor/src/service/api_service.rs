use std::sync::Arc;
use std::{any, ops::Deref};

use tokio::sync::RwLock;

use crate::{
    consistent_hash::{ConsistentHashRing, Hasher},
    registry::{self, RegistryClient},
};

#[derive(Clone)]
pub struct ApiRouter {
    pub user_service_list: Arc<RwLock<ConsistentHashRing>>,
    pub channel_service_list: Arc<RwLock<ConsistentHashRing>>,
    pub message_service_list: Arc<RwLock<ConsistentHashRing>>,
}

impl ApiRouter {
    const REPLICAS: usize = 100;

    const USER_SERVICE_PREFIX: &'static str = "user-service";
    const CHANNEL_SERVICE_PREFIX: &'static str = "channel-service";
    const MESSAGE_SERVICE_PREFIX: &'static str = "message-service";

    pub fn new(hasher: Option<Hasher>) -> Self {
        let hasher = hasher.unwrap_or(|key: &str| {
            use std::hash::Hasher as StdHasher;

            let mut default_hasher = twox_hash::XxHash64::with_seed(0);

            default_hasher.write(key.as_bytes());

            default_hasher.finish()
        });

        Self {
            user_service_list: Arc::new(RwLock::new(ConsistentHashRing::new(
                Self::REPLICAS,
                hasher,
            ))),
            channel_service_list: Arc::new(RwLock::new(ConsistentHashRing::new(
                Self::REPLICAS,
                hasher,
            ))),
            message_service_list: Arc::new(RwLock::new(ConsistentHashRing::new(
                Self::REPLICAS,
                hasher,
            ))),
        }
    }

    pub async fn update_user_service(&self, registry: &RegistryClient) -> anyhow::Result<()> {
        let services = registry
            .get_service_instances(Self::USER_SERVICE_PREFIX)
            .await?;

        let mut write_guard = self.user_service_list.write().await;

        write_guard.clear();

        services.iter().for_each(|instance| {
            let address = format!("{}:{}", instance.service.address, instance.service.port);
            write_guard.add_node(&address);
        });

        Ok(())
    }

    pub async fn update_channel_service(&self, registry: &RegistryClient) -> anyhow::Result<()> {
        let services = registry
            .get_service_instances(Self::CHANNEL_SERVICE_PREFIX)
            .await?;

        let mut write_guard = self.channel_service_list.write().await;

        write_guard.clear();

        services.iter().for_each(|instance| {
            let address = format!("{}:{}", instance.service.address, instance.service.port);
            write_guard.add_node(&address);
        });

        Ok(())
    }

    pub async fn update_message_service(&self, registry: &RegistryClient) -> anyhow::Result<()> {
        let services = registry
            .get_service_instances(Self::MESSAGE_SERVICE_PREFIX)
            .await?;

        let mut write_guard = self.message_service_list.write().await;

        write_guard.clear();

        services.iter().for_each(|instance| {
            let address = format!("{}:{}", instance.service.address, instance.service.port);
            write_guard.add_node(&address);
        });

        Ok(())
    }

    pub async fn pick_user_service(&self, key: &str) -> Option<String> {
        let read_guard = self.user_service_list.read().await;

        read_guard.get_node(key).map(|s| s.to_string())
    }

    pub async fn pick_channel_service(&self, key: &str) -> Option<String> {
        let read_guard = self.channel_service_list.read().await;

        read_guard.get_node(key).map(|s| s.to_string())
    }

    pub async fn pick_message_service(&self, key: &str) -> Option<String> {
        let read_guard = self.message_service_list.read().await;

        read_guard.get_node(key).map(|s| s.to_string())
    }
}
