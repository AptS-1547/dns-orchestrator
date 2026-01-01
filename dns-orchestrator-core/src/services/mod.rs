//! 业务逻辑服务层

mod account_bootstrap_service;
mod account_lifecycle_service;
mod account_metadata_service;
mod credential_management_service;
mod dns_service;
mod domain_service;
mod import_export_service;
mod migration_service;
mod provider_metadata_service;
mod toolbox;

pub use account_bootstrap_service::{AccountBootstrapService, RestoreResult};
pub use account_lifecycle_service::AccountLifecycleService;
pub use account_metadata_service::AccountMetadataService;
pub use credential_management_service::CredentialManagementService;
pub use dns_service::DnsService;
pub use domain_service::DomainService;
pub use import_export_service::ImportExportService;
pub use migration_service::{MigrationResult, MigrationService};
pub use provider_metadata_service::ProviderMetadataService;
pub use toolbox::ToolboxService;

use std::sync::Arc;

use dns_orchestrator_provider::DnsProvider;

use crate::error::{CoreError, CoreResult};
use crate::traits::{AccountRepository, CredentialStore, ProviderRegistry};
use crate::types::AccountStatus;

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

    /// 获取 Provider 实例
    pub async fn get_provider(&self, account_id: &str) -> CoreResult<Arc<dyn DnsProvider>> {
        self.provider_registry
            .get(account_id)
            .await
            .ok_or_else(|| CoreError::AccountNotFound(account_id.to_string()))
    }

    /// 标记账户为无效状态
    ///
    /// 当检测到凭证失效时调用此方法更新账户状态。
    pub async fn mark_account_invalid(&self, account_id: &str, error_msg: &str) {
        if let Err(e) = self
            .account_repository
            .update_status(
                account_id,
                AccountStatus::Error,
                Some(error_msg.to_string()),
            )
            .await
        {
            log::error!("Failed to mark account {account_id} as invalid: {e}");
            return;
        }
        log::warn!("Account {account_id} marked as invalid: {error_msg}");
    }
}
