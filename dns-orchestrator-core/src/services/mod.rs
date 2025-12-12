//! 业务逻辑服务层

mod account_service;
mod dns_service;
mod domain_service;
mod toolbox;

pub use account_service::{AccountService, RestoreResult};
pub use dns_service::DnsService;
pub use domain_service::DomainService;
pub use toolbox::ToolboxService;

use std::sync::Arc;

use crate::traits::{AccountRepository, CredentialStore, ProviderRegistry};

/// 服务上下文 - 持有所有依赖
///
/// 平台层需要创建此上下文，并注入平台特定的存储实现。
pub struct ServiceContext {
    /// 凭证存储
    pub credential_store: Arc<dyn CredentialStore>,
    /// 账户持久化仓库
    pub account_repository: Arc<dyn AccountRepository>,
    /// Provider 注册表
    pub provider_registry: Arc<dyn ProviderRegistry>,
}

impl ServiceContext {
    /// 创建服务上下文
    #[must_use]
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        account_repository: Arc<dyn AccountRepository>,
        provider_registry: Arc<dyn ProviderRegistry>,
    ) -> Self {
        Self {
            credential_store,
            account_repository,
            provider_registry,
        }
    }
}
