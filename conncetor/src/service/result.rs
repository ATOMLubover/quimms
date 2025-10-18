use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use tracing::trace;

pub type ServiceResult<T> = Result<ServiceValue<T>, ServiceError>;

pub fn succeed<T>() -> ServiceValue<T>
where
    T: Serialize,
{
    ServiceValue::<T>::default()
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        // Log the error for debugging purposes here,
        // so that we do not log it again in the match below.
        trace!("ServiceError occurred: {:?}", self);

        let (status, message) = match &self {
            ServiceError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
        };

        let body = serde_json::json!({
            "code": status.as_u16(),
            "message": message,
        });

        (status, Json(body)).into_response()
    }
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

impl<T> IntoResponse for ServiceValue<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status = self
            .code
            .try_into()
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::Json(self);

        (status, body).into_response()
    }
}
