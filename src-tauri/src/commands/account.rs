use tauri::State;

use crate::crypto;
use crate::error::DnsError;
use crate::providers::create_provider;
use crate::storage::AccountStore;
use crate::types::*;
use crate::AppState;

/// 列出所有账号
#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Account>>, DnsError> {
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
) -> Result<ApiResponse<Account>, DnsError> {
    let provider_type = match &request.provider {
        DnsProvider::Cloudflare => "cloudflare",
        DnsProvider::Aliyun => "aliyun",
        DnsProvider::Dnspod => "dnspod",
        DnsProvider::Huaweicloud => "huaweicloud",
    };

    // 1. 创建 provider 实例
    let provider = create_provider(provider_type, request.credentials.clone())?;

    // 2. 验证凭证
    let is_valid = provider.validate_credentials().await?;

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
            DnsError::CredentialError(e.to_string())
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
) -> Result<ApiResponse<()>, DnsError> {
    // 1. 检查账号是否存在
    let mut accounts = state.accounts.write().await;
    let index = accounts
        .iter()
        .position(|a| a.id == account_id)
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()))?;

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
pub async fn list_providers() -> Result<ApiResponse<Vec<ProviderMetadata>>, DnsError> {
    let providers = crate::providers::get_all_provider_metadata();
    Ok(ApiResponse::success(providers))
}

/// 导出账号（准备导出内容，返回 JSON 字符串）
#[tauri::command]
pub async fn export_accounts(
    state: State<'_, AppState>,
    request: ExportAccountsRequest,
) -> Result<ApiResponse<ExportAccountsResponse>, DnsError> {
    // 1. 获取选中账号的元数据
    let accounts = state.accounts.read().await;
    let selected_accounts: Vec<&Account> = accounts
        .iter()
        .filter(|a| request.account_ids.contains(&a.id))
        .collect();

    if selected_accounts.is_empty() {
        return Ok(ApiResponse::error("NO_ACCOUNTS", "没有选中任何账号"));
    }

    // 2. 加载凭证并构建导出数据
    let mut exported_accounts = Vec::new();
    for account in selected_accounts {
        let credentials = match state.credential_store.load(&account.id) {
            Ok(creds) => creds,
            Err(e) => {
                log::warn!("Failed to load credentials for {}: {}", account.id, e);
                continue;
            }
        };

        exported_accounts.push(ExportedAccount {
            id: uuid::Uuid::new_v4().to_string(), // 生成新 ID，避免导入时冲突
            name: account.name.clone(),
            provider: account.provider.clone(),
            created_at: account.created_at.clone(),
            updated_at: account.updated_at.clone(),
            credentials,
        });
    }

    // 3. 序列化账号数据
    let accounts_json = serde_json::to_value(&exported_accounts)
        .map_err(|e| DnsError::SerializationError(e.to_string()))?;

    // 4. 构建导出文件
    let now = chrono::Utc::now().to_rfc3339();
    let app_version = env!("CARGO_PKG_VERSION").to_string();

    let export_file = if request.encrypt {
        let password = request
            .password
            .as_ref()
            .ok_or_else(|| DnsError::ValidationError("加密导出需要提供密码".to_string()))?;

        let plaintext = serde_json::to_vec(&accounts_json)
            .map_err(|e| DnsError::SerializationError(e.to_string()))?;

        let (salt, nonce, ciphertext) = crypto::encrypt(&plaintext, password)
            .map_err(|e| DnsError::ImportExportError(e.to_string()))?;

        ExportFile {
            header: ExportFileHeader {
                version: 1,
                encrypted: true,
                salt: Some(salt),
                nonce: Some(nonce),
                exported_at: now,
                app_version,
            },
            data: serde_json::Value::String(ciphertext),
        }
    } else {
        ExportFile {
            header: ExportFileHeader {
                version: 1,
                encrypted: false,
                salt: None,
                nonce: None,
                exported_at: now,
                app_version,
            },
            data: accounts_json,
        }
    };

    // 5. 生成文件内容
    let content = serde_json::to_string_pretty(&export_file)
        .map_err(|e| DnsError::SerializationError(e.to_string()))?;

    let suggested_filename = format!(
        "dns-orchestrator-backup-{}.dnso",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );

    Ok(ApiResponse::success(ExportAccountsResponse {
        content,
        suggested_filename,
    }))
}

/// 预览导入文件
#[tauri::command]
pub async fn preview_import(
    state: State<'_, AppState>,
    content: String,
    password: Option<String>,
) -> Result<ApiResponse<ImportPreview>, DnsError> {
    // 1. 解析文件
    let export_file: ExportFile = serde_json::from_str(&content)
        .map_err(|e| DnsError::ImportExportError(format!("无效的导入文件: {}", e)))?;

    // 2. 检查版本
    if export_file.header.version > 1 {
        return Ok(ApiResponse::error(
            "UNSUPPORTED_VERSION",
            "不支持的文件版本",
        ));
    }

    // 3. 如果加密但未提供密码，返回需要密码的提示
    if export_file.header.encrypted && password.is_none() {
        return Ok(ApiResponse::success(ImportPreview {
            encrypted: true,
            account_count: 0,
            accounts: None,
        }));
    }

    // 4. 解密或直接解析账号数据
    let accounts: Vec<ExportedAccount> = if export_file.header.encrypted {
        let password = password.as_ref().unwrap();
        let ciphertext = export_file
            .data
            .as_str()
            .ok_or_else(|| DnsError::ImportExportError("无效的加密数据".to_string()))?;
        let salt = export_file
            .header
            .salt
            .as_ref()
            .ok_or_else(|| DnsError::ImportExportError("缺少加密盐值".to_string()))?;
        let nonce = export_file
            .header
            .nonce
            .as_ref()
            .ok_or_else(|| DnsError::ImportExportError("缺少加密 nonce".to_string()))?;

        let plaintext = crypto::decrypt(ciphertext, password, salt, nonce)
            .map_err(|_| DnsError::ImportExportError("解密失败，请检查密码是否正确".to_string()))?;

        serde_json::from_slice(&plaintext)
            .map_err(|e| DnsError::ImportExportError(format!("解析账号数据失败: {}", e)))?
    } else {
        serde_json::from_value(export_file.data)
            .map_err(|e| DnsError::ImportExportError(format!("解析账号数据失败: {}", e)))?
    };

    // 5. 检查与现有账号的冲突
    let existing_accounts = state.accounts.read().await;
    let existing_names: std::collections::HashSet<_> =
        existing_accounts.iter().map(|a| a.name.as_str()).collect();

    let preview_accounts: Vec<ImportPreviewAccount> = accounts
        .iter()
        .map(|a| ImportPreviewAccount {
            name: a.name.clone(),
            provider: a.provider.clone(),
            has_conflict: existing_names.contains(a.name.as_str()),
        })
        .collect();

    Ok(ApiResponse::success(ImportPreview {
        encrypted: export_file.header.encrypted,
        account_count: accounts.len(),
        accounts: Some(preview_accounts),
    }))
}

/// 执行导入
#[tauri::command]
pub async fn import_accounts(
    state: State<'_, AppState>,
    request: ImportAccountsRequest,
) -> Result<ApiResponse<ImportResult>, DnsError> {
    // 1. 解析和解密（逻辑与 preview_import 类似）
    let export_file: ExportFile = serde_json::from_str(&request.content)
        .map_err(|e| DnsError::ImportExportError(format!("无效的导入文件: {}", e)))?;

    let accounts: Vec<ExportedAccount> = if export_file.header.encrypted {
        let password = request
            .password
            .as_ref()
            .ok_or_else(|| DnsError::ImportExportError("加密文件需要提供密码".to_string()))?;
        let ciphertext = export_file
            .data
            .as_str()
            .ok_or_else(|| DnsError::ImportExportError("无效的加密数据".to_string()))?;
        let salt = export_file
            .header
            .salt
            .as_ref()
            .ok_or_else(|| DnsError::ImportExportError("缺少加密盐值".to_string()))?;
        let nonce = export_file
            .header
            .nonce
            .as_ref()
            .ok_or_else(|| DnsError::ImportExportError("缺少加密 nonce".to_string()))?;

        let plaintext = crypto::decrypt(ciphertext, password, salt, nonce)
            .map_err(|_| DnsError::ImportExportError("解密失败，请检查密码是否正确".to_string()))?;

        serde_json::from_slice(&plaintext)
            .map_err(|e| DnsError::ImportExportError(format!("解析账号数据失败: {}", e)))?
    } else {
        serde_json::from_value(export_file.data)
            .map_err(|e| DnsError::ImportExportError(format!("解析账号数据失败: {}", e)))?
    };

    // 2. 逐个导入账号
    let mut success_count = 0;
    let mut failures = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    for exported in accounts {
        // 2.1 创建 provider 实例验证凭证
        let provider_type = match &exported.provider {
            DnsProvider::Cloudflare => "cloudflare",
            DnsProvider::Aliyun => "aliyun",
            DnsProvider::Dnspod => "dnspod",
            DnsProvider::Huaweicloud => "huaweicloud",
        };

        let provider = match create_provider(provider_type, exported.credentials.clone()) {
            Ok(p) => p,
            Err(e) => {
                failures.push(ImportFailure {
                    name: exported.name.clone(),
                    reason: format!("创建 Provider 失败: {}", e),
                });
                continue;
            }
        };

        // 2.2 生成新的账号 ID
        let account_id = uuid::Uuid::new_v4().to_string();

        // 2.3 保存凭证到 Keychain
        if let Err(e) = state
            .credential_store
            .save(&account_id, &exported.credentials)
        {
            failures.push(ImportFailure {
                name: exported.name.clone(),
                reason: format!("保存凭证失败: {}", e),
            });
            continue;
        }

        // 2.4 注册 provider
        state.registry.register(account_id.clone(), provider).await;

        // 2.5 创建账号元数据
        let account = Account {
            id: account_id,
            name: exported.name,
            provider: exported.provider,
            created_at: now.clone(),
            updated_at: now.clone(),
            status: Some(AccountStatus::Active),
            error: None,
        };

        // 2.6 保存到内存
        state.accounts.write().await.push(account);
        success_count += 1;
    }

    // 3. 持久化账户元数据
    let accounts = state.accounts.read().await.clone();
    if let Err(e) = AccountStore::save_accounts(&state.app_handle, &accounts) {
        log::error!("Failed to persist accounts after import: {}", e);
    }

    Ok(ApiResponse::success(ImportResult {
        success_count,
        failures,
    }))
}
