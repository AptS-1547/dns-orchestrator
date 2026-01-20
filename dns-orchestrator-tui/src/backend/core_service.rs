//! 核心服务
//!
//! 封装 dns-orchestrator-core 的各种服务，
//! 提供给 TUI 层使用的统一接口

use std::sync::Arc;

use dns_orchestrator_core::services::{
    AccountBootstrapService, AccountLifecycleService, AccountMetadataService,
    CredentialManagementService, DnsService, DomainService, ProviderMetadataService,
    ServiceContext,
};
use dns_orchestrator_core::traits::InMemoryProviderRegistry;
use dns_orchestrator_core::CoreResult;

use super::account_repository::JsonAccountRepository;
use super::credential_service::KeyringCredentialStore;
use super::domain_metadata_repository::InMemoryDomainMetadataRepository;

/// TUI 核心服务
///
/// 持有所有业务服务的实例，提供给 UI 层调用
pub struct CoreService {
    /// 服务上下文（供 DomainService/DnsService 使用）
    ctx: Arc<ServiceContext>,
    /// 账号元数据服务
    metadata_service: Arc<AccountMetadataService>,
    /// 凭证管理服务
    credential_service: Arc<CredentialManagementService>,
}

impl CoreService {
    /// 创建核心服务实例
    pub fn new() -> Self {
        // 1. 创建基础依赖
        let credential_store = Arc::new(KeyringCredentialStore::new());
        let account_repository = Arc::new(JsonAccountRepository::new());
        let provider_registry = Arc::new(InMemoryProviderRegistry::new());
        let domain_metadata_repository = Arc::new(InMemoryDomainMetadataRepository::new());

        // 2. 创建 ServiceContext（供 DomainService/DnsService 使用）
        let ctx = Arc::new(ServiceContext::new(
            credential_store.clone(),
            account_repository.clone(),
            provider_registry.clone(),
            domain_metadata_repository,
        ));

        // 3. 创建服务实例
        let metadata_service = Arc::new(AccountMetadataService::new(account_repository));
        let credential_service = Arc::new(CredentialManagementService::new(
            credential_store,
            provider_registry,
        ));

        Self {
            ctx,
            metadata_service,
            credential_service,
        }
    }

    /// 初始化：恢复所有账号的 Provider 实例
    ///
    /// 应在应用启动时调用
    pub async fn initialize(&self) -> CoreResult<()> {
        let bootstrap_service = AccountBootstrapService::new(
            self.metadata_service.clone(),
            self.credential_service.clone(),
        );
        let result = bootstrap_service.restore_accounts().await?;

        if result.error_count > 0 {
            log::warn!(
                "部分账号恢复失败: {} 成功, {} 失败",
                result.success_count,
                result.error_count
            );
        }

        Ok(())
    }

    // ========== 账号管理 ==========

    /// 获取账号生命周期服务
    pub fn account_lifecycle(&self) -> AccountLifecycleService {
        AccountLifecycleService::new(
            self.metadata_service.clone(),
            self.credential_service.clone(),
        )
    }

    /// 获取账号元数据服务
    pub fn account_metadata(&self) -> Arc<AccountMetadataService> {
        self.metadata_service.clone()
    }

    // ========== 域名管理 ==========

    /// 获取域名服务
    pub fn domain(&self) -> DomainService {
        DomainService::new(self.ctx.clone())
    }

    // ========== DNS 记录管理 ==========

    /// 获取 DNS 记录服务
    pub fn dns(&self) -> DnsService {
        DnsService::new(self.ctx.clone())
    }

    // ========== 其他服务 ==========

    /// 获取 Provider 元数据服务
    pub fn provider_metadata(&self) -> ProviderMetadataService {
        ProviderMetadataService::new()
    }
}

impl Default for CoreService {
    fn default() -> Self {
        Self::new()
    }
}
