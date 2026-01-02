//! 账户启动恢复服务
//!
//! 负责应用启动时恢复账户状态，重建 Provider 实例

use std::sync::Arc;

use dns_orchestrator_provider::create_provider;

use crate::error::CoreResult;
use crate::types::AccountStatus;

use super::{AccountMetadataService, CredentialManagementService};

/// 账户恢复结果
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// 成功恢复的账户数
    pub success_count: usize,
    /// 恢复失败的账户数
    pub error_count: usize,
}

/// 账户启动恢复服务
pub struct AccountBootstrapService {
    metadata_service: Arc<AccountMetadataService>,
    credential_service: Arc<CredentialManagementService>,
}

impl AccountBootstrapService {
    /// 创建账户启动恢复服务实例
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

    /// 恢复账户（启动时调用）
    pub async fn restore_accounts(&self) -> CoreResult<RestoreResult> {
        let mut success_count = 0;
        let mut error_count = 0;

        // 1. 加载所有账户元数据
        let accounts = self.metadata_service.list_accounts().await?;

        // 2. 加载所有凭证
        let all_credentials = match self.credential_service.load_all_credentials().await {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("Failed to load credentials: {e}");
                // 标记所有账户为错误状态
                for account in &accounts {
                    if let Err(update_err) = self
                        .metadata_service
                        .update_status(&account.id, AccountStatus::Error, Some(e.to_string()))
                        .await
                    {
                        log::warn!(
                            "Failed to update status for account {}: {update_err}",
                            account.id
                        );
                    }
                }
                return Ok(RestoreResult {
                    success_count: 0,
                    error_count: accounts.len(),
                });
            }
        };

        // 3. 逐个恢复账户
        for account in &accounts {
            let Some(credentials) = all_credentials.get(&account.id) else {
                log::warn!("No credentials found for account: {}", account.id);
                if let Err(e) = self
                    .metadata_service
                    .update_status(
                        &account.id,
                        AccountStatus::Error,
                        Some("凭证不存在".to_string()),
                    )
                    .await
                {
                    log::warn!("Failed to update status for account {}: {e}", account.id);
                }
                error_count += 1;
                continue;
            };

            // 直接使用 ProviderCredentials 创建 provider
            let provider = match create_provider(credentials.clone()) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!(
                        "Failed to create provider for account {}: {}",
                        account.id,
                        e
                    );
                    if let Err(update_err) = self
                        .metadata_service
                        .update_status(
                            &account.id,
                            AccountStatus::Error,
                            Some(format!("创建 Provider 失败: {e}")),
                        )
                        .await
                    {
                        log::warn!(
                            "Failed to update status for account {}: {update_err}",
                            account.id
                        );
                    }
                    error_count += 1;
                    continue;
                }
            };

            // 注册 provider
            self.credential_service
                .register_provider(account.id.clone(), provider)
                .await;

            // 更新状态为 Active
            if let Err(e) = self
                .metadata_service
                .update_status(&account.id, AccountStatus::Active, None)
                .await
            {
                log::warn!("Failed to update status for account {}: {e}", account.id);
            }

            success_count += 1;
        }

        Ok(RestoreResult {
            success_count,
            error_count,
        })
    }
}
