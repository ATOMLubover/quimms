use serde::{Deserialize, Serialize};

use crate::model::dto::{
    CreateChannelReq, CreateChannelRsp, CreateMessageReq, CreateMessageRsp, GetUserInfoReq,
    GetUserInfoRsp, JoinChannelReq, JoinChannelRsp, ListChannelDetailsReq, ListChannelDetailsRsp,
    ListMessagesReq, ListMessagesRsp, LoginUserReq, LoginUserRsp, RegisterUserReq, RegisterUserRsp,
};

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
}
