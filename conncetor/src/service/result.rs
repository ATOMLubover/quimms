use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

pub type ServiceResult<T> = Result<ServiceValue<T>, ServiceError>;

pub fn succeed<T>() -> ServiceValue<T>
where
    T: Serialize,
{
    ServiceValue::<T>::default()
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Tonic transport error: {0}")]
    TonicTransportError(#[from] tonic::transport::Error),
    #[error("gRPC status error: {0}")]
    GprcStatusError(#[from] tonic::Status),
    #[error("Upstream unaccesible error")]
    UpstreamUnaccesibleError,
}

#[derive(Debug, Serialize)]
pub struct ServiceValue<T = ()>
where
    T: Serialize,
{
    pub code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[allow(dead_code)]
impl<T> ServiceValue<T>
where
    T: Serialize,
{
    pub fn default() -> Self {
        Self {
            code: StatusCode::OK.as_u16(),
            message: None,
            data: None,
        }
    }

    pub fn with_code(mut self, code: StatusCode) -> Self {
        self.code = code.into();

        self
    }

    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());

        self
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);

        self
    }
}
