use std::ops::Deref;
use std::sync::Arc;

use crossfire::MAsyncTx;
use dashmap::DashMap;

use crate::cache::CacheClient;
use crate::config::AppConfig;
use crate::registry::RegistryClient;

/// `AppState` is a cloneable wrapper around `AppStateInner` using `Arc`.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn new(config: AppConfig, cache: CacheClient, registry: RegistryClient) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                cache,
                registry,
                online_users: DashMap::new(),
            }),
        }
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// `AppStateInner` has not to be Clone because `AppState` is the one being cloned.
#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub config: AppConfig,
    pub cache: CacheClient,
    pub registry: RegistryClient,
    pub online_users: DashMap<String, MAsyncTx<String>>,
}
