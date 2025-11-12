use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceEntry {
    service: ServiceInfo,
}

impl ServiceEntry {
    pub fn new(service: ServiceInfo) -> Self {
        Self { service }
    }

    pub fn info(&self) -> &ServiceInfo {
        &self.service
    }

    pub fn service(&self) -> &ServiceInfo {
        &self.service
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInfo {
    #[serde(rename = "ID")]
    id: String,
    service: String,
    address: String,
    port: u16,
}

impl ServiceInfo {
    pub fn new(id: String, service: String, address: String, port: u16) -> Self {
        Self {
            id,
            service,
            address,
            port,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    pub fn name(&self) -> &str {
        &self.service
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HeathCheck {
    pub ttl: Duration,
    #[serde(rename = "CheckID")]
    pub check_id: String,
    pub name: String,
}

impl HeathCheck {
    pub fn new(ttl: Duration, check_id: String, name: String) -> Self {
        Self {
            ttl,
            check_id,
            name,
        }
    }

    pub fn ttl(&self) -> &Duration {
        &self.ttl
    }

    pub fn check_id(&self) -> &str {
        &self.check_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Registry {
    #[serde(rename = "ID")]
    id: String,
    name: String,
    address: String,
    port: u16,
    check: HeathCheck,
}

impl Registry {
    pub fn new(id: String, name: String, address: String, port: u16, check: HeathCheck) -> Self {
        Self {
            id,
            name,
            address,
            port,
            check,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn check(&self) -> &HeathCheck {
        &self.check
    }
}
