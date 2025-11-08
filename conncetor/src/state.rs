use std::sync::Arc;

use crossfire::MAsyncTx;
use dashmap::DashMap;
use tonic::transport::Channel;

use crate::cache::CacheClient;
use crate::config::AppConfig;
use crate::message::InnerMessage;
use crate::registry::ConsulClient;

/// `AppState` is a cloneable wrapper around `AppStateInner` using `Arc`.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    inner: Arc<Inner>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        cache: CacheClient,
        user_service: ConsulClient<Channel>,
        channel_service: ConsulClient<Channel>,
        message_service: ConsulClient<Channel>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                cache,
                user_service,
                channel_service,
                message_service,
                online_users: DashMap::new(),
            }),
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.inner.config
    }

    pub fn cache(&self) -> &CacheClient {
        &self.inner.cache
    }

    pub fn user_service(&self) -> &ConsulClient<Channel> {
        &self.inner.user_service
    }

    pub fn channel_service(&self) -> &ConsulClient<Channel> {
        &self.inner.channel_service
    }

    pub fn message_service(&self) -> &ConsulClient<Channel> {
        &self.inner.message_service
    }

    pub fn online_users(&self) -> &DashMap<String, MAsyncTx<InnerMessage>> {
        &self.inner.online_users
    }
}

/// `AppStateInner` has not to be Clone because `AppState` is the one being cloned.
#[derive(Debug)]
struct Inner {
    config: AppConfig,
    cache: CacheClient,
    user_service: ConsulClient<Channel>,
    channel_service: ConsulClient<Channel>,
    message_service: ConsulClient<Channel>,
    online_users: DashMap<String, MAsyncTx<InnerMessage>>,
}
