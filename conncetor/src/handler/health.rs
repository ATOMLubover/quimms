use crate::{handler::result::HttpResult, service};

pub async fn health_check() -> HttpResult<()> {
    service::health_check().await.into()
}
