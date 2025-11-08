use std::fmt::Debug;
use std::sync::Arc;
use std::sync::RwLock;

use dashmap::DashMap;
use tonic::transport::Channel;
use tracing::debug;
use tracing::error;

use crate::{
    consist_hash::{ConsistHashRing, Hasher},
    registry::ConsulClient,
};

#[derive(Clone)]
pub struct UpstreamRouter {
    registry: ConsulClient,

    user_svr_addr: Arc<RwLock<ConsistHashRing>>,
    user_srv_channels: DashMap<String, Channel>,

    channel_svr_addr: Arc<RwLock<ConsistHashRing>>,
    channel_srv_channels: DashMap<String, Channel>,

    message_svr_addr: Arc<RwLock<ConsistHashRing>>,
    message_srv_channels: DashMap<String, Channel>,
}

#[derive(Debug)]
pub enum Service {
    UserService,
    ChannelService,
    MessageService,
}

impl ToString for Service {
    fn to_string(&self) -> String {
        match self {
            Service::UserService => "user-service".to_string(),
            Service::ChannelService => "channel-service".to_string(),
            Service::MessageService => "message-service".to_string(),
        }
    }
}

impl UpstreamRouter {
    const REPLICAS: usize = 100;

    const DEFAULT_HASHER: fn(&str) -> u64 = |key: &str| {
        use std::hash::Hasher as StdHasher;

        let mut default_hasher = twox_hash::XxHash64::with_seed(0);

        default_hasher.write(key.as_bytes());

        default_hasher.finish()
    };

    pub fn new(registry: ConsulClient, hasher: Option<Hasher>) -> Self {
        let hasher = hasher.unwrap_or(Self::DEFAULT_HASHER);

        Self {
            registry,
            user_svr_addr: Arc::new(RwLock::new(ConsistHashRing::new(Self::REPLICAS, hasher))),
            user_srv_channels: DashMap::new(),
            channel_svr_addr: Arc::new(RwLock::new(ConsistHashRing::new(Self::REPLICAS, hasher))),
            channel_srv_channels: DashMap::new(),
            message_svr_addr: Arc::new(RwLock::new(ConsistHashRing::new(Self::REPLICAS, hasher))),
            message_srv_channels: DashMap::new(),
        }
    }

    pub async fn full_update(&mut self) -> anyhow::Result<()> {
        for service in &[
            Service::UserService,
            Service::ChannelService,
            Service::MessageService,
        ] {
            self.update_service_list(service).await?;
        }

        Ok(())
    }

    pub async fn update_service_list(&mut self, service: &Service) -> anyhow::Result<()> {
        let instances = self
            .registry
            .get_service_instances(&service.to_string())
            .await?;

        if instances.is_empty() {
            error!("No available instances for service {:?}", service);
            return Ok(());
        }

        debug!(
            "Try updating service list for {:?}, instances: {:?}",
            service, &instances
        );

        let mut new_ring = ConsistHashRing::new(Self::REPLICAS, Self::DEFAULT_HASHER);
        let new_map = DashMap::new();

        for inst in &instances {
            let address = format!("http://{}:{}", inst.service.address, inst.service.port);

            let channel = match Channel::from_shared(address.clone())?.connect().await {
                Ok(chan) => {
                    // We only reserve the channel if connection is successful.
                    chan
                }
                Err(err) => {
                    error!(
                        "Failed to connect to {:?} service instance {}: {}",
                        service, address, err
                    );
                    continue;
                }
            };

            match service {
                Service::UserService => {
                    new_ring.add_node(&address);
                    new_map.insert(address, channel);
                }
                Service::ChannelService => {
                    new_ring.add_node(&address);
                    new_map.insert(address, channel);
                }
                Service::MessageService => {
                    new_ring.add_node(&address);
                    new_map.insert(address, channel);
                }
            };
        }

        // Minimize the critical section by swapping in the new ring and map.
        match service {
            Service::UserService => {
                let mut ring = self.user_svr_addr.write().unwrap();
                *ring = new_ring;
                self.user_srv_channels = new_map;
            }
            Service::ChannelService => {
                let mut ring = self.channel_svr_addr.write().unwrap();
                *ring = new_ring;
                self.channel_srv_channels = new_map;
            }
            Service::MessageService => {
                let mut ring = self.message_svr_addr.write().unwrap();
                *ring = new_ring;
                self.message_srv_channels = new_map;
            }
        };

        Ok(())
    }

    pub async fn pick_service(&self, service: Service, key: &str) -> Option<Channel> {
        // Another trick to minimize the critical section.
        // We only hold ONE lock at the same time to avoid deadlock.
        match service {
            Service::UserService => {
                let addr;

                {
                    let hash_ring = self.user_svr_addr.read().unwrap();

                    addr = hash_ring.get_node(key).map(|s| s.to_string());
                }

                addr.and_then(|a| self.user_srv_channels.get(&a).map(|c| c.clone()))
            }
            Service::ChannelService => {
                let addr;

                {
                    let hash_ring = self.channel_svr_addr.read().unwrap();

                    addr = hash_ring.get_node(key).map(|s| s.to_string());
                }

                addr.and_then(|a| self.channel_srv_channels.get(&a).map(|c| c.clone()))
            }
            Service::MessageService => {
                let addr;

                {
                    let hash_ring = self.message_svr_addr.read().unwrap();

                    addr = hash_ring.get_node(key).map(|s| s.to_string());
                }

                addr.and_then(|a| self.message_srv_channels.get(&a).map(|c| c.clone()))
            }
        }
    }
}

impl Debug for UpstreamRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiRouter")
            .field("registry", &self.registry)
            .finish()
    }
}
