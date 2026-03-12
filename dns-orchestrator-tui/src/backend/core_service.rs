//! 核心服务
//!
//! 封装 dns-orchestrator-core 的各种服务，
//! 提供给 TUI 层使用的统一接口

use std::sync::Arc;

use dns_orchestrator_core::CoreResult;
use dns_orchestrator_core::services::{
    AccountService, DnsService, DomainMetadataService, DomainService, ProviderMetadataService,
    ServiceContext,
};
use dns_orchestrator_core::traits::InMemoryProviderRegistry;

use super::account_repository::JsonAccountRepository;
use super::credential_service::KeyringCredentialStore;
use super::domain_metadata_repository::InMemoryDomainMetadataRepository;

/// TUI 核心服务
///
/// 持有所有业务服务的实例，提供给 UI 层调用
pub struct CoreService {
    /// 服务上下文（供各服务使用）
    ctx: Arc<ServiceContext>,
    /// 域名元数据服务
    domain_metadata_service: Arc<DomainMetadataService>,
}

impl CoreService {
    /// 创建核心服务实例
    pub fn new() -> Self {
        // 1. 创建基础依赖
        let credential_store = Arc::new(KeyringCredentialStore::new());
        let account_repository = Arc::new(JsonAccountRepository::new());
        let provider_registry = Arc::new(InMemoryProviderRegistry::new());
        let domain_metadata_repository = Arc::new(InMemoryDomainMetadataRepository::new());

        // 2. 创建域名元数据服务
        let domain_metadata_service = Arc::new(DomainMetadataService::new(
            domain_metadata_repository.clone(),
        ));

        // 3. 创建 ServiceContext（供各服务使用）
        let ctx = Arc::new(ServiceContext::new(
            credential_store,
            account_repository,
            provider_registry,
            domain_metadata_repository,
        ));

        Self {
            ctx,
            domain_metadata_service,
        }
    }

    /// 初始化：恢复所有账号的 Provider 实例
    ///
    /// 应在应用启动时调用
    pub async fn initialize(&self) -> CoreResult<()> {
        let account_service = self.account();
        let result = account_service.restore_accounts().await?;

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

    /// 获取账号服务
    pub fn account(&self) -> AccountService {
        AccountService::new(self.ctx.clone())
    }

    // ========== 域名管理 ==========

    /// 获取域名服务
    pub fn domain(&self) -> DomainService {
        DomainService::new(self.ctx.clone(), self.domain_metadata_service.clone())
    }

    /// 获取域名元数据服务
    pub fn domain_metadata(&self) -> Arc<DomainMetadataService> {
        self.domain_metadata_service.clone()
    }

    // ========== DNS 记录管理 ==========

    /// 获取 DNS 记录服务
    pub fn dns(&self) -> DnsService {
        DnsService::new(self.ctx.clone())
    }

    // ========== 其他服务 ==========

    /// 获取 Provider 元数据服务
    #[allow(clippy::unused_self)]
    pub fn provider_metadata(&self) -> ProviderMetadataService {
        ProviderMetadataService::new()
    }
}

impl Default for CoreService {
    fn default() -> Self {
        Self::new()
    }
}
