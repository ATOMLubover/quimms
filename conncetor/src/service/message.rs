use crate::{
    model::dto::{
        CreateMessageRequest, CreateMessageResponse, ListMessagesRequest, ListMessagesResponse,
        MessageDetail,
    },
    service::{
        ServiceError, ServiceResult,
        message::message_service::message_service_client::MessageServiceClient, succeed,
    },
    upstream::{Service, UpstreamRouter},
};

mod message_service {
    tonic::include_proto!("message_service");
}

pub async fn create_message(
    request: CreateMessageRequest,
    upstreams: &UpstreamRouter,
) -> ServiceResult<CreateMessageResponse> {
    let server = upstreams
        .pick_service(Service::MessageService, &request.sender_id)
        .await
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?;

    let mut client = MessageServiceClient::new(server);

    let grpc_request = tonic::Request::new(message_service::CreateMessageRequest {
        sender_id: request.sender_id,
        channel_id: request.channel_id,
        content: request.content,
    });

    let grpc_response = client.create_message(grpc_request).await?.into_inner();

    Ok(succeed().with_data(CreateMessageResponse {
        message_id: grpc_response.message_id,
    }))
}

async fn list_channel_messages(
    request: ListMessagesRequest,
    upstreams: &UpstreamRouter,
) -> ServiceResult<ListMessagesResponse> {
    let server = upstreams
        .pick_service(Service::MessageService, &request.channel_id)
        .await
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?;

    let mut client = MessageServiceClient::new(server);

    let grpc_request = tonic::Request::new(message_service::ListChannelMessagesRequest {
        channel_id: request.channel_id,
        limit: request.limit as i32,
        latest_time: request.latest_time,
    });

    let grpc_response = client
        .list_channel_messages(grpc_request)
        .await?
        .into_inner();

    let messages: Vec<MessageDetail> = grpc_response
        .messages
        .into_iter()
        .map(|msg| MessageDetail {
            message_id: msg.message_id,
            sender_id: msg.sender_id,
            channel_id: msg.channel_id,
            content: msg.content,
            timestamp: msg.created_at,
        })
        .collect();

    Ok(succeed().with_data(ListMessagesResponse { messages }))
}
