pub mod dto {
    mod user {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct RegisterUserRequest {
            pub username: String,
            pub password: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct RegisterUserResponse {
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct LoginUserRequest {
            pub username: String,
            pub password: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct LoginUserResponse {
            pub token: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct GetUserInfoRequest {
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct GetUserInfoResponse {
            pub user_id: String,
            pub username: String,
            pub created_at: i64,
        }
    }

    mod channel {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CreateChannelRequest {
            pub name: String,
            pub creator_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CreateChannelResponse {
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
        pub struct ListChannelDetailsRequest {
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ListChannelDetailsResponse {
            pub channels: Vec<ChannelDetail>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct JoinChannelRequest {
            pub channel_id: String,
            pub user_id: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct JoinChannelResponse {
            pub channel_id: String,
            pub user_id: String,
        }
    }

    mod message {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CreateMessageRequest {
            pub channel_id: String,
            pub sender_id: String,
            pub content: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct CreateMessageResponse {
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
        pub struct ListMessagesRequest {
            pub channel_id: String,
            pub limit: usize,
            pub latest_time: i64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ListMessagesResponse {
            pub messages: Vec<MessageDetail>,
        }
    }

    pub use channel::*;
    pub use message::*;
    pub use user::*;
}
