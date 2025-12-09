mod commands;
mod credentials;
mod crypto;
mod error;
mod providers;
mod storage;
mod types;

use std::sync::Arc;

#[cfg(target_os = "android")]
use commands::updater;
use commands::{account, dns, domain, toolbox};
#[cfg(target_os = "android")]
use credentials::AndroidCredentialStore;
use credentials::CredentialStore;
#[cfg(not(target_os = "android"))]
use credentials::KeychainStore;
use providers::ProviderRegistry;
use storage::AccountStore;
use tauri::Manager;
use tokio::sync::RwLock;
use types::Account;

/// 应用全局状态
pub struct AppState {
    /// Provider 注册表
    pub registry: ProviderRegistry,
    /// 凭证存储
    pub credential_store: Arc<dyn CredentialStore>,
    /// 账号元数据 (不含凭证)
    pub accounts: RwLock<Vec<Account>>,
    /// App Handle (用于访问 Store)
    pub app_handle: tauri::AppHandle,
}

impl AppState {
    #[cfg(not(target_os = "android"))]
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            registry: ProviderRegistry::new(),
            credential_store: Arc::new(KeychainStore::new()),
            accounts: RwLock::new(Vec::new()),
            app_handle,
        }
    }

    #[cfg(target_os = "android")]
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            registry: ProviderRegistry::new(),
            credential_store: Arc::new(AndroidCredentialStore::new(app_handle.clone())),
            accounts: RwLock::new(Vec::new()),
            app_handle,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init());

    // 仅桌面端启用 updater
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    // Android 启用 Stronghold 和 APK Installer
    #[cfg(target_os = "android")]
    {
        builder = builder
            .plugin(
                tauri_plugin_stronghold::Builder::with_argon2(&std::path::PathBuf::from(
                    "stronghold_salt.txt",
                ))
                .build(),
            )
            .plugin(tauri_plugin_apk_installer::init());
    }

    let builder = builder.setup(|app| {
        // 创建 AppState（需要 AppHandle）
        let state = AppState::new(app.handle().clone());

        // 从持久化存储恢复账户
        if let Err(e) = restore_accounts(&state) {
            log::error!("Failed to restore accounts: {e}");
            // 不阻止应用启动，只记录错误
        }

        app.manage(state);
        Ok(())
    });

    #[cfg(not(target_os = "android"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        // Account commands
        account::list_accounts,
        account::create_account,
        account::delete_account,
        account::list_providers,
        account::export_accounts,
        account::preview_import,
        account::import_accounts,
        // Domain commands
        domain::list_domains,
        domain::get_domain,
        // DNS commands
        dns::list_dns_records,
        dns::create_dns_record,
        dns::update_dns_record,
        dns::delete_dns_record,
        dns::batch_delete_dns_records,
        // Toolbox commands
        toolbox::whois_lookup,
        toolbox::dns_lookup,
        toolbox::ip_lookup,
        toolbox::ssl_check,
    ]);

    #[cfg(target_os = "android")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        // Account commands
        account::list_accounts,
        account::create_account,
        account::delete_account,
        account::list_providers,
        account::export_accounts,
        account::preview_import,
        account::import_accounts,
        // Domain commands
        domain::list_domains,
        domain::get_domain,
        // DNS commands
        dns::list_dns_records,
        dns::create_dns_record,
        dns::update_dns_record,
        dns::delete_dns_record,
        dns::batch_delete_dns_records,
        // Toolbox commands
        toolbox::whois_lookup,
        toolbox::dns_lookup,
        toolbox::ip_lookup,
        toolbox::ssl_check,
        // Android updater commands
        updater::check_android_update,
        updater::download_apk,
        updater::install_apk,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 从持久化存储恢复账户
///
/// 流程：
/// 1. 从 Store 加载账户元数据
/// 2. 一次性从 Keychain 加载所有凭证（只访问一次 Keychain）
/// 3. 重建 Provider 实例
/// 4. 注册到 `ProviderRegistry`
fn restore_accounts(state: &AppState) -> crate::error::Result<()> {
    use crate::providers::create_provider;
    use crate::types::AccountStatus;

    // 1. 加载账户元数据
    let mut accounts = AccountStore::load_accounts(&state.app_handle)?;

    if accounts.is_empty() {
        log::info!("No accounts to restore");
        return Ok(());
    }

    log::info!("Restoring {} accounts...", accounts.len());

    // 2. 一次性加载所有凭证（只访问一次 Keychain）
    let all_credentials = match state.credential_store.load_all() {
        Ok(creds) => creds,
        Err(e) => {
            log::error!("Failed to load credentials from Keychain: {e}");
            // 标记所有账户为错误状态
            for account in &mut accounts {
                account.status = Some(AccountStatus::Error);
                account.error = Some(format!("凭证加载失败: {e}"));
            }
            futures::executor::block_on(async {
                let mut accounts_guard = state.accounts.write().await;
                *accounts_guard = accounts;
            });
            return Ok(());
        }
    };

    // 3. 遍历账户，恢复 Provider 实例
    let mut restored_count = 0;
    let mut failed_count = 0;

    for account in &mut accounts {
        // 3.1 从已加载的凭证中获取该账户的凭证
        let credentials = if let Some(creds) = all_credentials.get(&account.id) {
            creds.clone()
        } else {
            log::warn!(
                "No credentials found for account {}: credential not in store",
                account.id
            );
            account.status = Some(AccountStatus::Error);
            account.error = Some("凭证未找到".to_string());
            failed_count += 1;
            continue;
        };

        // 3.2 确定 provider 类型
        let provider_type = match &account.provider {
            crate::types::DnsProvider::Cloudflare => "cloudflare",
            crate::types::DnsProvider::Aliyun => "aliyun",
            crate::types::DnsProvider::Dnspod => "dnspod",
            crate::types::DnsProvider::Huaweicloud => "huaweicloud",
        };

        // 3.3 重建 Provider 实例
        let provider = match create_provider(provider_type, &credentials) {
            Ok(p) => p,
            Err(e) => {
                log::warn!(
                    "Failed to create provider for account {}: {}",
                    account.id,
                    e
                );
                account.status = Some(AccountStatus::Error);
                account.error = Some(format!("Provider 创建失败: {e}"));
                failed_count += 1;
                continue;
            }
        };

        // 3.4 注册到 ProviderRegistry（同步执行，因为在 setup 阶段）
        let registry = state.registry.clone();
        let account_id = account.id.clone();

        // 使用 futures 的 block_on 来同步执行异步注册
        futures::executor::block_on(async {
            registry.register(account_id, provider).await;
        });

        account.status = Some(AccountStatus::Active);
        restored_count += 1;
    }

    // 4. 更新内存中的账户列表（包含失败的账户）
    futures::executor::block_on(async {
        let mut accounts_guard = state.accounts.write().await;
        *accounts_guard = accounts;
    });

    log::info!("Account restoration complete: {restored_count} succeeded, {failed_count} failed");

    if restored_count == 0 && failed_count > 0 {
        log::error!("All accounts failed to restore. Please check Keychain access permissions.");
    }

    Ok(())
}
