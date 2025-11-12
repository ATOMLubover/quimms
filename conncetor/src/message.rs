use crate::model::dto::DispatchedMessage;
use crate::model::dto::{
    CreateChannelRsp, CreateMessageRsp, GetUserInfoRsp, JoinChannelRsp, ListChannelDetailsRsp,
    ListMessagesRsp, LoginUserRsp, RegisterUserRsp,
};

#[derive(Debug)]
pub enum ServiceMessage {
    Pong,
    DispatchMessage(DispatchedMessage),
    RegisterUserRsp(RegisterUserRsp),
    LoginUserRsp(LoginUserRsp),
    GetUserInfoRsp(GetUserInfoRsp),
    CreateChannelRsp(CreateChannelRsp),
    ListChannelDetailsRsp(ListChannelDetailsRsp),
    JoinChannelRsp(JoinChannelRsp),
    CreateMessageRsp(CreateMessageRsp),
    ListMessagesRsp(ListMessagesRsp),
}
