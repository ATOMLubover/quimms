use tonic::transport::Channel;

use crate::{
    model::dto::{
        CreateMessageReq, CreateMessageRsp, ListMessagesReq, ListMessagesRsp, MessageDetail,
    },
    registry::{ConsulRegistry, store::Store},
    service::{
        ServiceError, ServiceResult,
        message::message_service::message_service_client::MessageServiceClient, succeed,
    },
};

mod message_service {
    tonic::include_proto!("message_service");
}

pub async fn create_message(
    args: CreateMessageReq,
    registry: &ConsulRegistry<Channel>,
) -> ServiceResult<CreateMessageRsp> {
    let chan = registry
        .store()
        .read()
        .unwrap()
        .pick(&args.user_id)
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?
        .extra_data()
        .clone();

    let mut client: MessageServiceClient<Channel> = MessageServiceClient::new(chan);

    let grpc_request = tonic::Request::new(message_service::CreateMessageRequest {
        user_id: args.user_id,
        channel_id: args.channel_id,
        content: args.content,
    });

    let grpc_response = client.create_message(grpc_request).await?.into_inner();

    Ok(succeed().with_data(CreateMessageRsp {
        message_id: grpc_response.message_id,
    }))
}

pub async fn list_channel_messages(
    args: ListMessagesReq,
    registry: &ConsulRegistry<Channel>,
) -> ServiceResult<ListMessagesRsp> {
    let chan = registry
        .store()
        .read()
        .unwrap()
        .pick(&args.channel_id)
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?
        .extra_data()
        .clone();

    let mut client = MessageServiceClient::new(chan);

    let grpc_request = tonic::Request::new(message_service::ListChannelMessagesRequest {
        channel_id: args.channel_id,
        limit: args.limit as i32,
        latest_time: args.latest_time,
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

    Ok(succeed().with_data(ListMessagesRsp { messages }))
}
