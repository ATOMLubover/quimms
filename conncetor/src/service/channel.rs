mod channel_service {
    tonic::include_proto!("channel_service");
}

use crate::{
    model::dto::{
        ChannelDetail, ChannelMember, CreateChannelReq, CreateChannelRsp, JoinChannelReq,
        JoinChannelRsp, ListChannelDetailsReq, ListChannelDetailsRsp,
    },
    registry::{ConsulRegistry, store::Store},
    service::{
        ServiceError, ServiceResult,
        channel::channel_service::channel_service_client::ChannelServiceClient, succeed,
    },
};
use tonic::transport::Channel;

pub async fn create_channel(
    args: CreateChannelReq,
    registry: &ConsulRegistry<Channel>,
) -> ServiceResult<CreateChannelRsp> {
    let chan = registry
        .store()
        .read()
        .unwrap()
        .pick(&args.creator_id)
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?
        .extra_data()
        .clone();

    let mut client = ChannelServiceClient::new(chan);

    let grpc_request =
        tonic::Request::new(channel_service::CreateChannelRequest { name: args.name });

    let grpc_response = client.create_channel(grpc_request).await?.into_inner();

    Ok(succeed().with_data(CreateChannelRsp {
        channel_id: grpc_response.id,
        channel_name: grpc_response.name,
    }))
}

pub async fn join_channel(
    args: JoinChannelReq,
    registry: &ConsulRegistry<Channel>,
) -> ServiceResult<JoinChannelRsp> {
    let chan = registry
        .store()
        .read()
        .unwrap()
        .pick(&args.user_id)
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?
        .extra_data()
        .clone();

    let mut client = ChannelServiceClient::new(chan);

    let grpc_request = tonic::Request::new(channel_service::JoinChannelRequest {
        user_id: args.user_id,
        channel_id: args.channel_id,
    });

    let grpc_response = client.join_channel(grpc_request).await?.into_inner();

    Ok(succeed().with_data(JoinChannelRsp {
        channel_id: grpc_response.channel_id,
        user_id: grpc_response.user_id,
    }))
}

pub async fn list_user_channels(
    args: ListChannelDetailsReq,
    registry: &ConsulRegistry<Channel>,
) -> ServiceResult<ListChannelDetailsRsp> {
    let chan = registry
        .store()
        .read()
        .unwrap()
        .pick(&args.user_id)
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?
        .extra_data()
        .clone();

    let mut client = ChannelServiceClient::new(chan);

    let grpc_request = tonic::Request::new(channel_service::ListChannelDetailRequest {
        user_id: args.user_id,
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

    Ok(succeed().with_data(ListChannelDetailsRsp { channels }))
}
