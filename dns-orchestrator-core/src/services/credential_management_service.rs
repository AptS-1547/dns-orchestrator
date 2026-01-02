//! 凭证管理服务
//!
//! 负责凭证的验证、保存、加载、删除，以及 Provider 实例的注册和管理

use std::sync::Arc;

use dns_orchestrator_provider::{create_provider, DnsProvider, ProviderCredentials};

use crate::error::{CoreError, CoreResult};
use crate::traits::{CredentialStore, CredentialsMap, ProviderRegistry};

/// 凭证管理服务
pub struct CredentialManagementService {
    credential_store: Arc<dyn CredentialStore>,
    provider_registry: Arc<dyn ProviderRegistry>,
}

impl CredentialManagementService {
    /// 创建凭证管理服务实例
    #[must_use]
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        provider_registry: Arc<dyn ProviderRegistry>,
    ) -> Self {
        Self {
            credential_store,
            provider_registry,
        }
    }

    /// 验证凭证并创建 Provider 实例
    ///
    /// # v1.7.0 变更
    /// 直接接受 `ProviderCredentials`，无需手动转换
    pub async fn validate_and_create_provider(
        &self,
        credentials: &ProviderCredentials,
    ) -> CoreResult<Arc<dyn DnsProvider>> {
        // 1. 创建 Provider
        let provider = create_provider(credentials.clone())?;

        // 2. 验证凭证
        let is_valid = provider.validate_credentials().await?;
        if !is_valid {
            return Err(CoreError::InvalidCredentials(
                credentials.provider_type().to_string(),
            ));
        }

        Ok(provider)
    }

    /// 保存凭证
    ///
    /// # v1.7.0 变更
    /// 使用 `set()` 方法和 `ProviderCredentials` 类型
    pub async fn save_credentials(
        &self,
        account_id: &str,
        credentials: &ProviderCredentials,
    ) -> CoreResult<()> {
        self.credential_store.set(account_id, credentials).await
    }

    /// 加载凭证
    ///
    /// # v1.7.0 变更
    /// 使用 `get()` 方法，返回 `ProviderCredentials`
    pub async fn load_credentials(&self, account_id: &str) -> CoreResult<ProviderCredentials> {
        self.credential_store.get(account_id).await?.ok_or_else(|| {
            CoreError::CredentialError(format!("No credentials found for account: {account_id}"))
        })
    }

    /// 删除凭证
    ///
    /// # v1.7.0 变更
    /// 使用 `remove()` 方法
    pub async fn delete_credentials(&self, account_id: &str) -> CoreResult<()> {
        self.credential_store.remove(account_id).await
    }

    /// 加载所有凭证
    ///
    /// # v1.7.0 变更
    /// 返回 `CredentialsMap` (类型安全的映射)
    pub async fn load_all_credentials(&self) -> CoreResult<CredentialsMap> {
        self.credential_store.load_all().await
    }

    /// 注册 Provider 到 Registry
    pub async fn register_provider(&self, account_id: String, provider: Arc<dyn DnsProvider>) {
        self.provider_registry.register(account_id, provider).await;
    }

    /// 注销 Provider
    pub async fn unregister_provider(&self, account_id: &str) {
        self.provider_registry.unregister(account_id).await;
    }
}
