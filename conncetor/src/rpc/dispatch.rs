use tonic::{Request, Response, Status};

pub mod dispatcher {
    tonic::include_proto!("dispatch_service");
}

use dispatcher::{
    DispatchMessageRequest, DispatchMessageResponse, dispatch_service_server::DispatchService,
};

use crate::{
    message::ServiceMessage, model::dto::DispatchedMessage, registry::ConsulRegistry,
    state::AppState,
};

pub struct DispatchServer {
    app_state: AppState,
}

impl DispatchServer {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            app_state: app_state.clone(),
        }
    }
}

#[tonic::async_trait]
impl DispatchService for DispatchServer {
    async fn dispatch_message(
        &self,
        request: Request<DispatchMessageRequest>,
    ) -> Result<Response<DispatchMessageResponse>, Status> {
        let request = request.into_inner();

        let user_serv_snd = match self.app_state.online_users().get(&request.target_user_id) {
            Some(tx) => tx,
            None => {
                return Err(Status::not_found("Target user is not online"));
            }
        };

        match user_serv_snd
            .send(ServiceMessage::DispatchMessage(DispatchedMessage {
                message_id: request.message_id,
                user_id: request.user_id,
                channel_id: request.channel_id,
                content: request.content,
                timestamp: request.created_at,
            }))
            .await
        {
            Ok(_) => Ok(Response::new(DispatchMessageResponse { successful: true })),
            Err(err) => Err(Status::internal(format!(
                "Failed to send message to user channel: {}",
                err
            ))),
        }
    }
}
