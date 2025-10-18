use dispatched::dispatch_server::Dispatch;
use tonic::{Request, Response, Status, Streaming};

use crate::transfer::recv::dispatched::{DispatchRequest, DispatchResponse};

pub mod dispatched {
    tonic::include_proto!("dispatched");
}

#[derive(Debug, Default)]
pub struct DispatchedServerImpl {}

#[tonic::async_trait]
impl Dispatch for DispatchedServerImpl {
    async fn dispatch_message(
        &self,
        request: Request<Streaming<DispatchRequest>>,
    ) -> Result<Response<DispatchResponse>, Status> {
        unimplemented!()
    }
}
