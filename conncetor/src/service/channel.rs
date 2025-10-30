mod channel_service {
    tonic::include_proto!("channel_service");
}

use crate::{
    model::dto::{
        ChannelDetail, ChannelMember, CreateChannelRequest, CreateChannelResponse,
        JoinChannelRequest, JoinChannelResponse, ListChannelDetailsRequest,
        ListChannelDetailsResponse,
    },
    service::{
        ServiceError, ServiceResult,
        channel::channel_service::channel_service_client::ChannelServiceClient, succeed,
    },
    upstream::{Service, UpstreamRouter},
};

pub async fn create_channel(
    request: CreateChannelRequest,
    upstreams: &UpstreamRouter,
) -> ServiceResult<CreateChannelResponse> {
    let server = upstreams
        .pick_service(Service::ChannelService, &request.creator_id)
        .await
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?;

    let mut client = ChannelServiceClient::new(server);

    let grpc_request =
        tonic::Request::new(channel_service::CreateChannelRequest { name: request.name });

    let grpc_response = client.create_channel(grpc_request).await?.into_inner();

    Ok(succeed().with_data(CreateChannelResponse {
        channel_id: grpc_response.id,
        channel_name: grpc_response.name,
    }))
}

pub async fn join_channel(
    request: JoinChannelRequest,
    upstreams: &UpstreamRouter,
) -> ServiceResult<JoinChannelResponse> {
    let server = upstreams
        .pick_service(Service::ChannelService, &request.user_id)
        .await
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?;

    let mut client = ChannelServiceClient::new(server);

    let grpc_request = tonic::Request::new(channel_service::JoinChannelRequest {
        user_id: request.user_id,
        channel_id: request.channel_id,
    });

    let grpc_response = client.join_channel(grpc_request).await?.into_inner();

    Ok(succeed().with_data(JoinChannelResponse {
        channel_id: grpc_response.channel_id,
        user_id: grpc_response.user_id,
    }))
}

pub async fn list_user_channels(
    request: ListChannelDetailsRequest,
    upstreams: &UpstreamRouter,
) -> ServiceResult<ListChannelDetailsResponse> {
    let server = upstreams
        .pick_service(Service::ChannelService, &request.user_id)
        .await
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?;

    let mut client = ChannelServiceClient::new(server);

    let grpc_request = tonic::Request::new(channel_service::ListChannelDetailRequest {
        user_id: request.user_id,
    });

    let grpc_response = client
        .list_channel_details(grpc_request)
        .await?
        .into_inner();

    let channels = grpc_response
        .channels
        .into_iter()
        .map(|chan_detail| ChannelDetail {
            channel_id: chan_detail.id,
            channel_name: chan_detail.name,
            members: chan_detail
                .members
                .into_iter()
                .map(|m| ChannelMember {
                    user_id: m.user_id,
                    joined_at: m.joined_at,
                })
                .collect(),
        })
        .collect();

    Ok(succeed().with_data(ListChannelDetailsResponse { channels }))
}
