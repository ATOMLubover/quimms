use reqwest::{Client, Url};

use crate::registry::upstream::ServiceInstance;

mod upstream;

#[derive(Debug)]
pub struct RegistryClient {
    base_url: Url,
    client: Client,
}

impl RegistryClient {
    pub fn new() -> anyhow::Result<Self> {
        let registry_url = std::env::var("CONSUL_ADDRESS")?;

        let registry_url = Url::parse(&registry_url)?;

        Ok(Self {
            base_url: registry_url,
            client: Client::new(),
        })
    }

    pub async fn get_service_instances(
        &self,
        service_prefix: &str,
    ) -> anyhow::Result<Vec<ServiceInstance>> {
        let url = self
            .base_url
            .join(&format!("/v1/health/service/{}", service_prefix))?;

        // Fetch only healthy instances.
        let resp = self.client.get(url).query("passing").send().await?;

        let instances: Vec<ServiceInstance> = resp.json().await?;

        Ok(instances)
    }
}
