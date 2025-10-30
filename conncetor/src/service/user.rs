mod user_service {
    tonic::include_proto!("user_service");
}

use crate::model::dto::{
    GetUserInfoResponse, LoginUserRequest, LoginUserResponse, RegisterUserRequest,
    RegisterUserResponse,
};
// TODO: Do not create a new client for each request. Implement connection pooling.
use crate::service::user::user_service::user_service_client::UserServiceClient;
use crate::service::{ServiceError, succeed};
use crate::{
    service::ServiceResult,
    upstream::{Service, UpstreamRouter},
};

pub async fn register_user(
    request: RegisterUserRequest,
    upstreams: &UpstreamRouter,
) -> ServiceResult<RegisterUserResponse> {
    let server = upstreams
        .pick_service(Service::UserService, &request.username)
        .await
        .ok_or_else(|| {
            ServiceError::UpstreamUnaccesibleError
        })?;

    let mut client = UserServiceClient::new(server);

    let grpc_request = tonic::Request::new(user_service::RegisterUserRequest {
        nickname: request.username,
        password: request.password,
    });

    let response = client.register_user(grpc_request).await?;

    let grpc_response = response.into_inner();

    Ok(succeed().with_data(RegisterUserResponse {
        user_id: grpc_response.user_id,
    }))
}

pub async fn login_user(
    request: LoginUserRequest,
    upstreams: &UpstreamRouter,
) -> ServiceResult<LoginUserResponse> {
    let server = upstreams
        .pick_service(Service::UserService, &request.username)
        .await
        .ok_or_else(|| {
            ServiceError::UpstreamUnaccesibleError
        })?;

    let mut client = UserServiceClient::new(server);

    let grpc_request = tonic::Request::new(user_service::LoginUserRequest {
        nickname: request.username,
        password: request.password,
    });

    let response = client.login_user(grpc_request).await?;

    let grpc_response = response.into_inner();

    Ok(succeed().with_data(LoginUserResponse {
        token: grpc_response.token,
    }))
}

pub async fn get_user_info(
    user_id: &str,
    upstreams: &UpstreamRouter,
) -> ServiceResult<GetUserInfoResponse> {
    let server = upstreams
        .pick_service(Service::UserService, user_id)
        .await
        .ok_or_else(|| ServiceError::UpstreamUnaccesibleError)?;

    let mut client = UserServiceClient::new(server);

    let grpc_request = tonic::Request::new(user_service::GetUserInfoRequest {
        user_id: user_id.to_string(),
    });

    let response = client.get_user_info(grpc_request).await?;

    let grpc_response = response.into_inner();

    Ok(succeed().with_data(GetUserInfoResponse {
        user_id: grpc_response.user_id,
        username: grpc_response.nickname,
        created_at: grpc_response.created_at,
    }))
}
