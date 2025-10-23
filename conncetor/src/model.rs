pub mod dto {
    mod user {
        pub struct RegisterUserRequest {
            pub username: String,
            pub password: String,
        }

        pub struct RegisterUserResponse {
            pub user_id: String,
        }

        pub struct LoginUserRequest {
            pub username: String,
            pub password: String,
        }

        pub struct LoginUserResponse {
            pub token: String,
        }

        pub struct GetUserInfoRequest {
            pub user_id: String,
        }

        pub struct GetUserInfoResponse {
            pub user_id: String,
            pub username: String,
            pub created_at: i64,
        }
    }

    mod channel {
        pub struct CreateChannelRequest {
            pub name: String,
            pub creator_id: String,
        }

        pub struct CreateChannelResponse {
            pub channel_id: String,
        }

        pub struct ChannelMember {
            pub user_id: String,
            pub joined_at: i64,
        }

        pub struct ChannelDetail {
            pub channel_id: String,
            pub channel_name: String,
            pub members: Vec<ChannelMember>,
        }

        pub struct ListChannelDetailsRequest {
            pub user_id: String,
        }

        pub struct ListChannelDetailsResponse {
            pub channels: Vec<ChannelDetail>,
        }

        pub struct JoinChannelRequest {
            pub channel_id: String,
            pub user_id: String,
        }

        pub struct JoinChannelResponse {
            pub channel_id: String,
            pub user_id: String,
        }
    }

    mod message {}

    use channel::*;
    use message::*;
    use user::*;
}
