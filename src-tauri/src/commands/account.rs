use tauri::State;

use crate::error::DnsError;
use crate::providers::create_provider;
use crate::storage::AccountStore;
use crate::types::*;
use crate::AppState;

/// 列出所有账号
#[tauri::command]
pub async fn list_accounts(state: State<'_, AppState>) -> Result<ApiResponse<Vec<Account>>, String> {
    let accounts = state.accounts.read().await.clone();
    Ok(ApiResponse::success(accounts))
}

/// 创建新账号
/// 1. 验证凭证
/// 2. 保存凭证到 Keychain
/// 3. 注册 Provider 实例
/// 4. 保存账号元数据
#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    request: CreateAccountRequest,
) -> Result<ApiResponse<Account>, String> {
    let provider_type = match &request.provider {
        DnsProvider::Cloudflare => "cloudflare",
        DnsProvider::Aliyun => "aliyun",
        DnsProvider::Dnspod => "dnspod",
        DnsProvider::Huaweicloud => "huaweicloud",
    };

    // 1. 创建 provider 实例
    let provider = create_provider(provider_type, request.credentials.clone())
        .map_err(|e| e.to_string())?;

    // 2. 验证凭证
    let is_valid = provider
        .validate_credentials()
        .await
        .map_err(|e| e.to_string())?;

    if !is_valid {
        return Ok(ApiResponse::error("INVALID_CREDENTIALS", "凭证验证失败"));
    }

    // 3. 生成账号 ID
    let account_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // 4. 保存凭证到 Keychain
    log::info!("Saving credentials to Keychain for account: {}", account_id);
    state
        .credential_store
        .save(&account_id, &request.credentials)
        .map_err(|e| {
            log::error!("Failed to save credentials to Keychain: {}", e);
            e.to_string()
        })?;
    log::info!("Credentials saved successfully to Keychain");

    // 5. 注册 provider 到 registry
    state.registry.register(account_id.clone(), provider).await;

    // 6. 创建账号元数据
    let account = Account {
        id: account_id,
        name: request.name,
        provider: request.provider,
        created_at: now.clone(),
        updated_at: now,
        status: Some(crate::types::AccountStatus::Active),
        error: None,
    };

    // 7. 保存账号元数据到内存
    state.accounts.write().await.push(account.clone());

    // 8. 持久化账户元数据到 Store
    let accounts = state.accounts.read().await.clone();
    if let Err(e) = AccountStore::save_accounts(&state.app_handle, &accounts) {
        log::error!("Failed to persist account to store: {}", e);
        // 不回滚，只记录错误（账户已在内存和 Keychain 中）
    }

    Ok(ApiResponse::success(account))
}

/// 删除账号
/// 1. 注销 Provider
/// 2. 删除凭证
/// 3. 删除账号元数据
#[tauri::command]
pub async fn delete_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<ApiResponse<()>, String> {
    // 1. 检查账号是否存在
    let mut accounts = state.accounts.write().await;
    let index = accounts
        .iter()
        .position(|a| a.id == account_id)
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()).to_string())?;

    // 2. 注销 provider
    state.registry.unregister(&account_id).await;

    // 3. 删除凭证 (忽略错误，凭证可能不存在)
    let _ = state.credential_store.delete(&account_id);

    // 4. 删除账号元数据（内存）
    accounts.remove(index);

    // 5. 从持久化存储中删除
    let accounts_clone = accounts.clone();
    drop(accounts); // 释放锁

    if let Err(e) = AccountStore::delete_account(&state.app_handle, &account_id, &accounts_clone) {
        log::error!("Failed to delete account from store: {}", e);
        // 不影响删除操作的成功
    }

    Ok(ApiResponse::success(()))
}

/// 获取所有支持的提供商列表
#[tauri::command]
pub async fn list_providers() -> Result<ApiResponse<Vec<ProviderMetadata>>, String> {
    let providers = crate::providers::get_all_provider_metadata();
    Ok(ApiResponse::success(providers))
}
