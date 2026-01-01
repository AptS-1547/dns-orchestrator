//! 账户生命周期服务
//!
//! 负责协调账户的完整 CRUD 操作，包括元数据、凭证和 Provider 的协同管理

use std::sync::Arc;

use chrono::Utc;

use crate::error::{CoreError, CoreResult};
use crate::types::{
    Account, AccountStatus, BatchDeleteFailure, BatchDeleteResult, CreateAccountRequest,
    UpdateAccountRequest,
};

use super::{AccountMetadataService, CredentialManagementService};

/// 账户生命周期服务
pub struct AccountLifecycleService {
    metadata_service: Arc<AccountMetadataService>,
    credential_service: Arc<CredentialManagementService>,
}

impl AccountLifecycleService {
    /// 创建账户生命周期服务实例
    #[must_use]
    pub fn new(
        metadata_service: Arc<AccountMetadataService>,
        credential_service: Arc<CredentialManagementService>,
    ) -> Self {
        Self {
            metadata_service,
            credential_service,
        }
    }

    /// 创建账户
    ///
    /// 完整流程：验证凭证 -> 保存凭证 -> 注册 Provider -> 保存元数据
    /// 如果保存元数据失败，会自动清理已保存的凭证和已注册的 Provider
    ///
    /// # v1.7.0 变更
    /// `request.credentials` 已经是 `ProviderCredentials` 类型，无需调用 `from_map()` 转换
    pub async fn create_account(&self, request: CreateAccountRequest) -> CoreResult<Account> {
        // 1. 验证凭证
        let provider = self
            .credential_service
            .validate_and_create_provider(&request.credentials)
            .await?;

        // 2. 生成账号 ID
        let account_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        // 3. 保存凭证
        log::info!("Saving credentials for account: {account_id}");
        self.credential_service
            .save_credentials(&account_id, &request.credentials)
            .await?;
        log::info!("Credentials saved successfully");

        // 4. 注册 provider
        self.credential_service
            .register_provider(account_id.clone(), provider)
            .await;

        // 5. 创建账号元数据
        let account = Account {
            id: account_id.clone(),
            name: request.name,
            provider: request.provider,
            created_at: now,
            updated_at: now,
            status: Some(AccountStatus::Active),
            error: None,
        };

        // 6. 保存元数据，失败时 cleanup
        if let Err(e) = self.metadata_service.save_account(&account).await {
            log::error!("Failed to save account metadata, cleaning up: {e}");
            // Cleanup: 删除凭证和注销 provider
            if let Err(cleanup_err) = self
                .credential_service
                .delete_credentials(&account_id)
                .await
            {
                log::warn!("Cleanup: failed to delete credentials for {account_id}: {cleanup_err}");
            }
            self.credential_service
                .unregister_provider(&account_id)
                .await;
            return Err(e);
        }

        Ok(account)
    }

    /// 更新账户
    ///
    /// 支持更新账户名称和/或凭证
    /// 如果更新凭证，会重新验证并重新注册 Provider
    ///
    /// # v1.7.0 变更
    /// `request.credentials` 已经是 `Option<ProviderCredentials>` 类型，无需调用 `from_map()` 转换
    pub async fn update_account(&self, request: UpdateAccountRequest) -> CoreResult<Account> {
        // 1. 获取现有账户
        let mut account = self
            .metadata_service
            .get_account(&request.id)
            .await?
            .ok_or_else(|| CoreError::AccountNotFound(request.id.clone()))?;

        // 2. 如果提供了新凭证，验证并更新
        if let Some(ref new_credentials) = request.credentials {
            // 2.1 验证凭证
            let new_provider = self
                .credential_service
                .validate_and_create_provider(new_credentials)
                .await?;

            // 2.2 更新凭证存储
            log::info!("Updating credentials for account: {}", request.id);
            self.credential_service
                .save_credentials(&request.id, new_credentials)
                .await?;

            // 2.3 重新注册 provider（先注册新的，避免竞态条件）
            self.credential_service
                .register_provider(request.id.clone(), new_provider)
                .await;
            self.credential_service
                .unregister_provider(&request.id)
                .await;

            // 2.4 更新状态为 Active（凭证验证成功）
            account.status = Some(AccountStatus::Active);
            account.error = None;
        }

        // 3. 更新名称（如果提供）
        if let Some(new_name) = request.name {
            account.name = new_name;
        }

        // 4. 更新时间戳
        account.updated_at = Utc::now();

        // 5. 保存更新后的账户
        self.metadata_service.save_account(&account).await?;

        Ok(account)
    }

    /// 删除账户
    ///
    /// 流程：先删除元数据，再清理内存和凭证（避免出现"幽灵账户"）
    pub async fn delete_account(&self, account_id: &str) -> CoreResult<()> {
        // 1. 检查账户存在
        self.metadata_service
            .get_account(account_id)
            .await?
            .ok_or_else(|| CoreError::AccountNotFound(account_id.to_string()))?;

        // 2. 先删除账号元数据（关键：如果这步失败，后续步骤不会执行，避免幽灵账户）
        self.metadata_service.delete_account(account_id).await?;

        // 3. 注销 provider（内存操作，不会失败）
        self.credential_service
            .unregister_provider(account_id)
            .await;

        // 4. 删除凭证（即使失败也只记录警告，因为元数据已删除，用户不会看到这个账户）
        if let Err(e) = self.credential_service.delete_credentials(account_id).await {
            log::warn!("Failed to delete credentials for {account_id}: {e}");
        }

        Ok(())
    }

    /// 批量删除账户
    pub async fn batch_delete_accounts(
        &self,
        account_ids: Vec<String>,
    ) -> CoreResult<BatchDeleteResult> {
        let mut success_count = 0;
        let mut failures = Vec::new();

        for account_id in account_ids {
            match self.delete_account(&account_id).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    failures.push(BatchDeleteFailure {
                        record_id: account_id,
                        reason: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchDeleteResult {
            success_count,
            failed_count: failures.len(),
            failures,
        })
    }
}
