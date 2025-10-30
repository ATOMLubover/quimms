use crate::service::result::{ServiceResult, succeed};

pub async fn health_check() -> ServiceResult<()> {
    // Add some dependency checks here if needed in the future.
    Ok(succeed().with_message("Health response from server."))
}
