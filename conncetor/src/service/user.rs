mod user_service {
    tonic::include_proto!("user_service");
}

use crate::model::dto::{
    GetUserInfoReq, GetUserInfoRsp, LoginUserReq, LoginUserRsp, RegisterUserReq, RegisterUserRsp,
};
use crate::registry::store::Store;
// TODO: Do not create a new client for each request. Implement connection pooling.
use crate::service::user::user_service::user_service_client::UserServiceClient;
use crate::service::{ServiceError, succeed};
use crate::{registry::ConsulRegistry, service::ServiceResult};
use tonic::transport::Channel;

pub async fn register_user(
    args: RegisterUserReq,
    registry: &ConsulRegistry<Channel>,
) -> ServiceResult<RegisterUserRsp> {
    let chan = registry
        .store()
        .read()
        .unwrap()
        .pick(&args.username)
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?
        .extra_data()
        .clone();

    let mut client = UserServiceClient::new(chan);

    let grpc_request = tonic::Request::new(user_service::RegisterUserRequest {
        nickname: args.username,
        password: args.password,
    });

    let response = client.register_user(grpc_request).await?;

    let grpc_response = response.into_inner();

    Ok(succeed().with_data(RegisterUserRsp {
        user_id: grpc_response.user_id,
    }))
}

pub async fn login_user(
    args: LoginUserReq,
    registry: &ConsulRegistry<Channel>,
) -> ServiceResult<LoginUserRsp> {
    let chan = registry
        .store()
        .read()
        .unwrap()
        .pick(&args.username)
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?
        .extra_data()
        .clone();

    let mut client = UserServiceClient::new(chan);

    let grpc_request = tonic::Request::new(user_service::LoginUserRequest {
        nickname: args.username,
        password: args.password,
    });

    let response = client.login_user(grpc_request).await?;

    let grpc_response = response.into_inner();

    Ok(succeed().with_data(LoginUserRsp {
        token: grpc_response.token,
    }))
}

pub async fn get_user_info(
    args: GetUserInfoReq,
    registry: &ConsulRegistry<Channel>,
) -> ServiceResult<GetUserInfoRsp> {
    let chan = registry
        .store()
        .read()
        .unwrap()
        .pick(&args.user_id)
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?
        .extra_data()
        .clone();

    let mut client = UserServiceClient::new(chan);

    let grpc_request = tonic::Request::new(user_service::GetUserInfoRequest {
        user_id: args.user_id,
    });

    let response = client.get_user_info(grpc_request).await?;

    let grpc_response = response.into_inner();

    Ok(succeed().with_data(GetUserInfoRsp {
        user_id: grpc_response.user_id,
        username: grpc_response.nickname,
        created_at: grpc_response.created_at,
    }))
}
