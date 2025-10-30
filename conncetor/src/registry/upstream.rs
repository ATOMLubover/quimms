use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInstance {
    pub service: ServiceInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInfo {
    pub service: String,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HealthCheck {
    #[serde(rename = "HTTP")]
    pub http: String,
    pub interval: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RegisterService {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub health: HealthCheck,
}
