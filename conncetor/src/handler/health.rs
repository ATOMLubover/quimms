use axum::response::IntoResponse;

use crate::service;

pub async fn health_check() -> impl IntoResponse {
    service::health_check().await
}
