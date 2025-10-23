use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInstance {
    pub service: ServiceInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInfo {
    pub service: String,
    pub address: String,
    pub port: u16,
}
