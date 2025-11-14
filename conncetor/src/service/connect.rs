use std::future::Future;
use std::ops::ControlFlow;

use anyhow::anyhow;
use axum::body::Bytes;
use axum::extract::ws;
use crossfire::MAsyncTx;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use tracing::error;

use crate::message::ServiceMessage;

use crate::model::dto::{
    CreateChannelReq, CreateChannelRsp, CreateMessageReq, CreateMessageRsp, DispatchedMessage,
    GetUserInfoReq, GetUserInfoRsp, JoinChannelReq, JoinChannelRsp, ListChannelDetailsReq,
    ListChannelDetailsRsp, ListMessagesReq, ListMessagesRsp, LoginUserReq, LoginUserRsp,
    RegisterUserReq, RegisterUserRsp,
};
use crate::registry::ConsulRegistry;
use crate::service::{self, ServiceValue};
use crate::service::user::{register_user, login_user, get_user_info};
use crate::service::channel::{create_channel, join_channel, list_user_channels};
use crate::service::message::{create_message, list_channel_messages};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ReqMessage {
    RegisterUser(RegisterUserReq),
    LoginUser(LoginUserReq),
    GetUserInfo(GetUserInfoReq),
    CreateChannel(CreateChannelReq),
    ListChannelDetails(ListChannelDetailsReq),
    JoinChannel(JoinChannelReq),
    CreateMessage(CreateMessageReq),
    ListMessages(ListMessagesReq),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum RspMessage {
    RegisterUser(RegisterUserRsp),
    LoginUser(LoginUserRsp),
    GetUserInfo(GetUserInfoRsp),
    CreateChannel(CreateChannelRsp),
    ListChannelDetails(ListChannelDetailsRsp),
    JoinChannel(JoinChannelRsp),
    CreateMessage(CreateMessageRsp),
    ListMessages(ListMessagesRsp),
    DispatchMessage(DispatchedMessage),
}

/// Handle a service message and convert it to a WebSocket message.
/// It may comes from gRPC server functions.
pub async fn handle_serv_message(serv_message: ServiceMessage) -> Option<ws::Message> {
    let response: RspMessage = match serv_message {
        ServiceMessage::Pong => return Some(ws::Message::Pong(Bytes::default())),
        ServiceMessage::DispatchMessage(msg) => RspMessage::DispatchMessage(msg),
        ServiceMessage::RegisterUserRsp(rsp) => RspMessage::RegisterUser(rsp),
        ServiceMessage::LoginUserRsp(rsp) => RspMessage::LoginUser(rsp),
        ServiceMessage::GetUserInfoRsp(rsp) => RspMessage::GetUserInfo(rsp),
        ServiceMessage::CreateChannelRsp(rsp) => RspMessage::CreateChannel(rsp),
        ServiceMessage::ListChannelDetailsRsp(rsp) => RspMessage::ListChannelDetails(rsp),
        ServiceMessage::JoinChannelRsp(rsp) => RspMessage::JoinChannel(rsp),
        ServiceMessage::CreateMessageRsp(rsp) => RspMessage::CreateMessage(rsp),
        ServiceMessage::ListMessagesRsp(rsp) => RspMessage::ListMessages(rsp),
    };

    let text_content = match serde_json::to_string(&response) {
        Ok(text) => text,
        Err(err) => {
            tracing::error!("Failed to serialize response message: {}", err);
            return None;
        }
    };

    Some(ws::Message::Text(text_content.into()))
}

pub async fn handle_websock_message(
    user_id: &str,
    user_serv_snd: &MAsyncTx<ServiceMessage>,
    user_registry: &ConsulRegistry<Channel>,
    channel_registry: &ConsulRegistry<Channel>,
    message_registry: &ConsulRegistry<Channel>,
    websock_message: ws::Message,
) -> ControlFlow<anyhow::Result<()>> {
    let text_content = match websock_message {
        ws::Message::Text(text) => text,
        ws::Message::Binary(_) => {
            return ControlFlow::Break(Err(anyhow::anyhow!("Binary messages are not supported")));
        }
        ws::Message::Close(_) => {
            return ControlFlow::Break(Ok(()));
        }
        ws::Message::Ping(_) => match user_serv_snd.send(ServiceMessage::Pong).await {
            Ok(_) => return ControlFlow::Continue(()),
            Err(err) => {
                return ControlFlow::Break(Err(anyhow::anyhow!(
                    "Failed to handle Ping message for user_id {}: {}",
                    user_id,
                    err
                )));
            }
        },
        ws::Message::Pong(_) => return ControlFlow::Continue(()),
    };

    let request: ReqMessage = match serde_json::from_str(&text_content) {
        Ok(req) => req,
        Err(err) => {
            return ControlFlow::Break(Err(anyhow::anyhow!(
                "Failed to parse request message: {}",
                err
            )));
        }
    };

    match request {
        ReqMessage::RegisterUser(req) => {
            let serv_result = match register_user(req, user_registry).await {
                Ok(val) => {
                    user_serv_snd
                        .send(ServiceMessage::RegisterUserRsp(val.data.unwrap()))
                        .await
                        .map_err(|err| {
                            anyhow!(
                                "Failed to send RegisterUserResponse to user service channel: {}, user_id: {}",
                                err,
                                user_id
                            )
                        })
                }
                Err(err) => {
                    error!("Error registering user for user_id {}: {}", user_id, err);
                    
                    return ControlFlow::Continue(());
                }
            };

            match serv_result {
                Ok(_) => {},
                Err(err) =>{
                    error!("Failed to send RegisterUserResponse for user_id {}: {}", user_id, err);
                },
            };

            ControlFlow::Continue(())
        }
        ReqMessage::LoginUser(req) => {
            let serv_result = match login_user(req, user_registry).await {
                Ok(val) => {
                    user_serv_snd
                        .send(ServiceMessage::LoginUserRsp(val.data.unwrap()))
                        .await
                        .map_err(|err| {
                            anyhow!(
                                "Failed to send LoginUserResponse to user service channel: {}, user_id: {}",
                                err,
                                user_id
                            )
                        })
                }
                Err(err) => {
                    error!("Error logging in user for user_id {}: {}", user_id, err);
                    return ControlFlow::Continue(());
                }
            };

            match serv_result {
                Ok(_) => {},
                Err(err) => {
                    error!("Failed to send LoginUserResponse for user_id {}: {}", user_id, err);
                },
            };

            ControlFlow::Continue(())
        }
        ReqMessage::GetUserInfo(req) => {
            let serv_result = match get_user_info(req, user_registry).await {
                Ok(val) => {
                    user_serv_snd
                        .send(ServiceMessage::GetUserInfoRsp(val.data.unwrap()))
                        .await
                        .map_err(|err| {
                            anyhow!(
                                "Failed to send GetUserInfoResponse to user service channel: {}, user_id: {}",
                                err,
                                user_id
                            )
                        })
                }
                Err(err) => {
                    error!("Error getting user info for user_id {}: {}", user_id, err);
                    return ControlFlow::Continue(());
                }
            };

            match serv_result {
                Ok(_) => {},
                Err(err) => {
                    error!("Failed to send GetUserInfoResponse for user_id {}: {}", user_id, err);
                },
            };

            ControlFlow::Continue(())
        }
        ReqMessage::CreateChannel(req) => {
            let serv_result = match create_channel(req, channel_registry).await {
                Ok(val) => {
                    user_serv_snd
                        .send(ServiceMessage::CreateChannelRsp(val.data.unwrap()))
                        .await
                        .map_err(|err| {
                            anyhow!(
                                "Failed to send CreateChannelResponse to user service channel: {}, user_id: {}",
                                err,
                                user_id
                            )
                        })
                }
                Err(err) => {
                    error!("Error creating channel for user_id {}: {}", user_id, err);
                    return ControlFlow::Continue(());
                }
            };

            match serv_result {
                Ok(_) => {},
                Err(err) => {
                    error!("Failed to send CreateChannelResponse for user_id {}: {}", user_id, err);
                },
            };

            ControlFlow::Continue(())
        }
        ReqMessage::ListChannelDetails(req) => {
            let serv_result = match list_user_channels(req, channel_registry).await {
                Ok(val) => {
                    user_serv_snd
                        .send(ServiceMessage::ListChannelDetailsRsp(val.data.unwrap()))
                        .await
                        .map_err(|err| {
                            anyhow!(
                                "Failed to send ListChannelDetailsResponse to user service channel: {}, user_id: {}",
                                err,
                                user_id
                            )
                        })
                }
                Err(err) => {
                    error!("Error listing channel details for user_id {}: {}", user_id, err);
                    return ControlFlow::Continue(());
                }
            };

            match serv_result {
                Ok(_) => {},
                Err(err) => {
                    error!("Failed to send ListChannelDetailsResponse for user_id {}: {}", user_id, err);
                },
            };

            ControlFlow::Continue(())
        }
        ReqMessage::JoinChannel(req) => {
            let serv_result = match join_channel(req, channel_registry).await {
                Ok(val) => {
                    user_serv_snd
                        .send(ServiceMessage::JoinChannelRsp(val.data.unwrap()))
                        .await
                        .map_err(|err| {
                            anyhow!(
                                "Failed to send JoinChannelResponse to user service channel: {}, user_id: {}",
                                err,
                                user_id
                            )
                        })
                }
                Err(err) => {
                    error!("Error joining channel for user_id {}: {}", user_id, err);
                    return ControlFlow::Continue(());
                }
            };

            match serv_result {
                Ok(_) => {},
                Err(err) => {
                    error!("Failed to send JoinChannelResponse for user_id {}: {}", user_id, err);
                },
            };

            ControlFlow::Continue(())
        }
        ReqMessage::CreateMessage(req) => {
            let serv_result = match create_message(req, message_registry).await {
                Ok(val) => {
                    user_serv_snd
                        .send(ServiceMessage::CreateMessageRsp(val.data.unwrap()))
                        .await
                        .map_err(|err| {
                            anyhow!(
                                "Failed to send CreateMessageResponse to user service channel: {}, user_id: {}",
                                err,
                                user_id
                            )
                        })
                }
                Err(err) => {
                    error!("Error creating message for user_id {}: {}", user_id, err);
                    return ControlFlow::Continue(());
                }
            };

            match serv_result {
                Ok(_) => {},
                Err(err) => {
                    error!("Failed to send CreateMessageResponse for user_id {}: {}", user_id, err);
                },
            };

            ControlFlow::Continue(())
        }
        ReqMessage::ListMessages(req) => {
            let serv_result = match list_channel_messages(req, message_registry).await {
                Ok(val) => {
                    user_serv_snd
                        .send(ServiceMessage::ListMessagesRsp(val.data.unwrap()))
                        .await
                        .map_err(|err| {
                            anyhow!(
                                "Failed to send ListMessagesResponse to user service channel: {}, user_id: {}",
                                err,
                                user_id
                            )
                        })
                }
                Err(err) => {
                    error!("Error listing messages for user_id {}: {}", user_id, err);
                    return ControlFlow::Continue(());
                }
            };

            match serv_result {
                Ok(_) => {},
                Err(err) => {
                    error!("Failed to send ListMessagesResponse for user_id {}: {}", user_id, err);
                },
            };

            ControlFlow::Continue(())
        }
    }
}
