pub mod dto {
    mod user {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct RegisterUserReq {
            pub username: String,
            pub password: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct RegisterUserRsp {
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct LoginUserReq {
            pub username: String,
            pub password: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct LoginUserRsp {
            pub token: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct GetUserInfoReq {
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct GetUserInfoRsp {
            pub user_id: String,
            pub username: String,
            pub created_at: i64,
        }
    }

    mod channel {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CreateChannelReq {
            pub name: String,
            pub creator_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CreateChannelRsp {
            pub channel_id: String,
            pub channel_name: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ChannelMember {
            pub user_id: String,
            pub joined_at: i64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ChannelDetail {
            pub channel_id: String,
            pub channel_name: String,
            pub members: Vec<ChannelMember>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ListChannelDetailsReq {
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ListChannelDetailsRsp {
            pub channels: Vec<ChannelDetail>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct JoinChannelReq {
            pub channel_id: String,
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct JoinChannelRsp {
            pub channel_id: String,
            pub user_id: String,
        }
    }

    mod message {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CreateMessageReq {
            pub channel_id: String,
            pub user_id: String,
            pub content: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CreateMessageRsp {
            pub message_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct MessageDetail {
            pub message_id: String,
            pub channel_id: String,
            pub sender_id: String,
            pub content: String,
            pub timestamp: i64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ListMessagesReq {
            pub channel_id: String,
            pub limit: usize,
            pub latest_time: i64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ListMessagesRsp {
            pub messages: Vec<MessageDetail>,
        }
    }

    mod rpc {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct DispatchedMessage {
            pub message_id: String,
            pub channel_id: String,
            pub user_id: String,
            pub content: String,
            pub timestamp: i64,
        }
    }

    pub use channel::*;
    pub use message::*;
    pub use rpc::*;
    pub use user::*;
}
