//! 凭证格式迁移服务（v1.7.0）
//!
//! 负责将旧格式凭证（HashMap<String, `HashMap`<String, String>>）迁移到新格式（ProviderCredentials）

use dns_orchestrator_provider::{ProviderCredentials, ProviderType};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{CoreError, CoreResult};
use crate::traits::{AccountRepository, CredentialStore};

/// 迁移服务
pub struct MigrationService {
    credential_store: Arc<dyn CredentialStore>,
    account_repository: Arc<dyn AccountRepository>,
}

impl MigrationService {
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        account_repository: Arc<dyn AccountRepository>,
    ) -> Self {
        Self {
            credential_store,
            account_repository,
        }
    }

    /// 检查并执行迁移（如果需要）
    ///
    /// 返回迁移结果，如果已是新格式则返回 `NotNeeded`
    pub async fn migrate_if_needed(&self) -> CoreResult<MigrationResult> {
        // 尝试加载凭证
        match self.credential_store.load_all().await {
            Ok(_) => {
                log::info!("凭证已是新格式，无需迁移");
                Ok(MigrationResult::NotNeeded)
            }
            Err(CoreError::MigrationRequired) => {
                log::info!("检测到旧格式凭证，开始迁移...");
                self.perform_migration().await
            }
            Err(e) => Err(e),
        }
    }

    /// 执行迁移
    ///
    /// 注意：备份逻辑已在 Tauri 层实现（src-tauri/src/lib.rs），
    /// 在调用 `migrate_if_needed()` 之前执行。
    async fn perform_migration(&self) -> CoreResult<MigrationResult> {
        // 1. 加载原始 JSON（备份已在调用此方法前由 Tauri 层完成）
        let raw_json = self.credential_store.load_raw_json().await?;

        // 2. 解析旧格式
        let old_creds: HashMap<String, HashMap<String, String>> =
            serde_json::from_str(&raw_json)
                .map_err(|e| CoreError::MigrationFailed(format!("解析旧格式失败: {e}")))?;

        if old_creds.is_empty() {
            log::info!("旧凭证为空，无需迁移");
            return Ok(MigrationResult::NotNeeded);
        }

        // 3. 获取账户 provider 信息
        let accounts = self.account_repository.find_all().await?;
        let account_providers: HashMap<String, ProviderType> =
            accounts.into_iter().map(|a| (a.id, a.provider)).collect();

        // 4. 转换凭证
        let mut new_creds = HashMap::new();
        let mut failed_accounts = Vec::new();

        for (account_id, old_cred_map) in old_creds {
            if let Some(provider) = account_providers.get(&account_id) {
                match ProviderCredentials::from_map(provider, &old_cred_map) {
                    Ok(provider_creds) => {
                        new_creds.insert(account_id.clone(), provider_creds);
                    }
                    Err(e) => {
                        log::warn!("账户 {account_id} 凭证转换失败: {e}");
                        failed_accounts.push((account_id, format!("转换失败: {e}")));
                    }
                }
            } else {
                log::warn!("找不到账户 {account_id} 的元数据，跳过迁移");
                failed_accounts.push((account_id, "账户元数据缺失".to_string()));
            }
        }

        // 5. 保存新格式
        if !new_creds.is_empty() {
            self.credential_store.save_all(&new_creds).await?;
            log::info!(
                "凭证迁移完成：成功 {} 个，失败 {} 个",
                new_creds.len(),
                failed_accounts.len()
            );
        }

        Ok(MigrationResult::Success {
            migrated_count: new_creds.len(),
            failed_accounts,
        })
    }

    // 注意：backup_credentials 方法已移除
    // 备份逻辑现在在 Tauri 层实现（src-tauri/src/lib.rs），
    // 因为 MigrationService 位于平台无关的 Core 层，不应访问文件系统。
}

/// 迁移结果
#[derive(Debug)]
pub enum MigrationResult {
    /// 不需要迁移（已是新格式或空数据）
    NotNeeded,

    /// 迁移成功
    Success {
        /// 成功迁移的账户数量
        migrated_count: usize,
        /// 失败的账户列表 (`account_id`, `error_reason`)
        failed_accounts: Vec<(String, String)>,
    },
}
