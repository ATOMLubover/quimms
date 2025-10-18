use reqwest::{Client, Url};

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
}
