use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    service_id: String,
    service_name: String,

    http_host: String,
    http_port: u16,

    grpc_host: String,
    grpc_port: u16,
    refresh_ttl_secs: u64,

    consul_host: String,
    consul_port: u16,
}

impl AppConfig {
    pub fn try_from_file(path: Option<&str>) -> anyhow::Result<Self> {
        let path = path.unwrap_or("app_config.json");

        let content = std::fs::read_to_string(path)?;

        let config: AppConfig = serde_json::from_str(&content)?;

        Ok(config)
    }

    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn http_host(&self) -> &str {
        &self.http_host
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn grpc_host(&self) -> &str {
        &self.grpc_host
    }

    pub fn grpc_port(&self) -> u16 {
        self.grpc_port
    }

    pub fn consul_host(&self) -> &str {
        &self.consul_host
    }

    pub fn consul_port(&self) -> u16 {
        self.consul_port
    }
}
