use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
    time::Duration,
};

use futures::future::join_all;
use reqwest::{Client, Url};
use tracing::warn;

use crate::registry::{
    model::{Registry, ServiceEntry},
    store::{ConsistHashStore, ServiceData, Store},
};

pub mod model;
pub mod store;

pub struct ConsulRegistry<T, S = ConsistHashStore<T>>
where
    T: Clone + Debug + Send + 'static,
    S: Store<Extra = T> + Debug + Send + 'static,
{
    base_url: Url,
    http_cli: Client,
    service_prefix: String,
    store: Arc<RwLock<S>>,
}

impl<T, S> ConsulRegistry<T, S>
where
    T: Clone + Debug + Send + 'static,
    S: Store<Extra = T> + Debug + Send + 'static,
{
    const UPDATE_INTERVAL_SECS: u64 = 30;

    pub fn new(consul_addr: &str, service_prefix: &str, store: S) -> anyhow::Result<Self> {
        let consul_addr = Url::parse(consul_addr)?;

        Ok(Self {
            base_url: consul_addr,
            http_cli: Client::new(),
            service_prefix: service_prefix.to_string(),
            store: Arc::new(RwLock::new(store)),
        })
    }

    pub fn store(&self) -> &Arc<RwLock<S>> {
        &self.store
    }

    pub async fn update_store<F, Fut>(&self, transformer: F) -> anyhow::Result<()>
    where
        F: Fn(ServiceEntry) -> Fut,
        Fut: Future<Output = T> + Send,
    {
        let client = self.http_cli.clone();

        let url = self
            .base_url
            .join(&format!(
                "/v1/health/service/{}?passing=true",
                &self.service_prefix
            ))
            .map_err(|err| anyhow::anyhow!("Failed to construct URL: {}", err))?;

        let res = client.get(url).send().await?;

        let services: Vec<ServiceEntry> = res.json().await?;

        let datas = join_all(services.into_iter().map(|entry| {
            let extra_data_fut = transformer(entry.clone());

            async move {
                let extra_data = extra_data_fut.await;

                ServiceData::new(entry, extra_data)
            }
        }))
        .await;

        self.store.write().unwrap().update(datas);

        Ok(())
    }

    pub fn spawn_update_store<F, Fut>(&self, transformer: F) -> anyhow::Result<()>
    where
        F: Fn(ServiceEntry) -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send,
    {
        let client = self.http_cli.clone();

        let store = self.store.clone();

        let url = self
            .base_url
            .join(&format!(
                "/v1/health/service/{}?passing=true",
                &self.service_prefix
            ))
            .map_err(|err| anyhow::anyhow!("Failed to construct URL: {}", err))?;

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(Self::UPDATE_INTERVAL_SECS));

            loop {
                interval.tick().await;

                let res = match client.get(url.clone()).send().await {
                    Ok(res) => res,
                    Err(err) => {
                        warn!("Failed to fetch services from Consul: {}", err);
                        continue;
                    }
                };

                let services: Vec<ServiceEntry> = match res.json().await {
                    Ok(services) => services,
                    Err(err) => {
                        warn!("Failed to parse services from Consul response: {}", err);
                        continue;
                    }
                };

                let datas = join_all(services.into_iter().map(|entry| {
                    let extra_data_fut = transformer(entry.clone());

                    async move {
                        let extra_data = extra_data_fut.await;

                        ServiceData::new(entry, extra_data)
                    }
                }))
                .await;

                store.write().unwrap().update(datas);
            }
        });

        Ok(())
    }

    pub async fn register(&self, service: Registry, ttl: Duration) -> anyhow::Result<()> {
        let url = self
            .base_url
            .join("/v1/agent/service/register")
            .map_err(|err| anyhow::anyhow!("Failed to construct URL: {}", err))?;

        let _ = self
            .http_cli
            .put(url)
            .json(&service)
            .send()
            .await?
            .error_for_status()?;

        self.spawn_refresh_ttl(service.check().check_id().to_string(), ttl)
            .await?;

        Ok(())
    }

    async fn spawn_refresh_ttl(&self, check_id: String, ttl: Duration) -> anyhow::Result<()> {
        let client = self.http_cli.clone();

        let url = self
            .base_url
            .join(&format!("/v1/agent/check/update/{}", &check_id))
            .map_err(|err| anyhow::anyhow!("Failed to construct URL: {}", err))?;

        tokio::spawn(async move {
            let mut ttl_timer = tokio::time::interval(ttl / 2);

            loop {
                ttl_timer.tick().await;

                let body = serde_json::json!({
                    "Status": "passing"
                });

                let res = match client.put(url.clone()).json(&body).send().await {
                    Ok(res) => res,
                    Err(err) => {
                        warn!("Failed to send TTL update for check {}: {}", check_id, err);
                        continue;
                    }
                };

                if let Err(err) = res.error_for_status() {
                    warn!(
                        "Received error response for TTL update for check {}: {}",
                        check_id, err
                    );
                }
            }
        });

        Ok(())
    }
}

impl<T, S> Debug for ConsulRegistry<T, S>
where
    T: Clone + Debug + Send + 'static,
    S: Store<Extra = T> + Debug + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsulClient")
            .field("base_url", &self.base_url)
            .field("http_cli", &self.http_cli)
            .field("store", &self.store)
            .finish()
    }
}
