use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crate::{
    consist_hash::{ConsistHashRing, Hasher},
    registry::model::ServiceEntry,
};

/// `ServiceData` encapsulates a service instance along with its associated extra data.
#[derive(Clone, Debug)]
pub struct ServiceData<T>
where
    T: Clone + Debug,
{
    entry: Arc<ServiceEntry>,
    extra_data: T,
}

impl<T> ServiceData<T>
where
    T: Clone + Debug,
{
    pub fn new(instance: ServiceEntry, extra_data: T) -> Self {
        Self {
            entry: Arc::new(instance),
            extra_data,
        }
    }

    pub fn entry(&self) -> &ServiceEntry {
        &self.entry
    }

    pub fn extra_data(&self) -> &T {
        &self.extra_data
    }
}

pub trait Store: Send + Sync {
    type Extra: Clone + Debug + Send;

    fn pick(&self, key: &str) -> Option<ServiceData<Self::Extra>>;
    fn list(&self) -> Vec<ServiceData<Self::Extra>>;
    fn update(&mut self, datas: Vec<ServiceData<Self::Extra>>);
    fn clear(&mut self);
}

#[derive(Debug)]
pub struct ConsistHashStore<T>
where
    T: Clone + Debug + Send + Sync,
{
    ring: ConsistHashRing,
    replicas: usize,
    hasher: Hasher,

    instances: HashMap<String, ServiceData<T>>,
}

impl<T> ConsistHashStore<T>
where
    T: Clone + Debug + Send + Sync,
{
    pub fn new(replicas: usize, hasher: Hasher) -> Self {
        Self {
            replicas,
            hasher,
            ring: ConsistHashRing::new(replicas, hasher),
            instances: HashMap::new(),
        }
    }
}

impl<T> Store for ConsistHashStore<T>
where
    T: Clone + Debug + Send + Sync,
{
    type Extra = T;

    fn pick(&self, key: &str) -> Option<ServiceData<T>> {
        self.ring
            .get_node(key)
            .and_then(|node_id| self.instances.get(node_id).cloned())
    }

    fn list(&self) -> Vec<ServiceData<T>> {
        self.instances.values().cloned().collect()
    }

    fn update(&mut self, datas: Vec<ServiceData<T>>) {
        let mut new_ring = ConsistHashRing::new(self.replicas, self.hasher);
        let mut new_instances = HashMap::new();

        for data in datas {
            let node_id = data.entry().info().id().to_string();

            new_ring.add_node(&node_id);
            new_instances.insert(node_id, data);
        }

        self.ring = new_ring;
        self.instances = new_instances;
    }

    fn clear(&mut self) {
        self.ring = ConsistHashRing::new(self.replicas, self.hasher);
        self.instances.clear();
    }
}
