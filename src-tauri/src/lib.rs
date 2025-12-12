mod adapters;
mod commands;
mod error;
mod types;

use std::sync::Arc;

#[cfg(target_os = "android")]
use commands::updater;
use commands::{account, dns, domain, toolbox};
use tauri::Manager;

use adapters::{TauriAccountRepository, TauriCredentialStore};
use dns_orchestrator_core::services::{
    AccountService, DnsService, DomainService, ServiceContext, ToolboxService,
};
use dns_orchestrator_core::traits::InMemoryProviderRegistry;

/// 应用全局状态
pub struct AppState {
    /// 服务上下文
    pub ctx: Arc<ServiceContext>,
    /// 账户服务
    pub account_service: AccountService,
    /// 域名服务
    pub domain_service: DomainService,
    /// DNS 服务
    pub dns_service: DnsService,
    /// 工具箱服务
    pub toolbox_service: ToolboxService,
}

impl AppState {
    #[cfg(not(target_os = "android"))]
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        // 创建适配器
        let credential_store = Arc::new(TauriCredentialStore::new());
        let account_repository = Arc::new(TauriAccountRepository::new(app_handle));
        let provider_registry = Arc::new(InMemoryProviderRegistry::new());

        // 创建服务上下文
        let ctx = Arc::new(ServiceContext::new(
            credential_store,
            account_repository,
            provider_registry,
        ));

        // 创建各服务
        let account_service = AccountService::new(Arc::clone(&ctx));
        let domain_service = DomainService::new(Arc::clone(&ctx));
        let dns_service = DnsService::new(Arc::clone(&ctx));
        let toolbox_service = ToolboxService::new();

        Self {
            ctx,
            account_service,
            domain_service,
            dns_service,
            toolbox_service,
        }
    }

    #[cfg(target_os = "android")]
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        // 创建适配器（Android 使用带 AppHandle 的版本）
        let credential_store = Arc::new(TauriCredentialStore::new(app_handle.clone()));
        let account_repository = Arc::new(TauriAccountRepository::new(app_handle));
        let provider_registry = Arc::new(InMemoryProviderRegistry::new());

        // 创建服务上下文
        let ctx = Arc::new(ServiceContext::new(
            credential_store,
            account_repository,
            provider_registry,
        ));

        // 创建各服务
        let account_service = AccountService::new(Arc::clone(&ctx));
        let domain_service = DomainService::new(Arc::clone(&ctx));
        let dns_service = DnsService::new(Arc::clone(&ctx));
        let toolbox_service = ToolboxService::new();

        Self {
            ctx,
            account_service,
            domain_service,
            dns_service,
            toolbox_service,
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

        // 使用 tokio runtime 来执行异步恢复
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        let result = rt.block_on(state.account_service.restore_accounts());

        match result {
            Ok(restore_result) => {
                log::info!(
                    "Account restoration complete: {} succeeded, {} failed",
                    restore_result.success_count,
                    restore_result.error_count
                );
            }
            Err(e) => {
                log::error!("Failed to restore accounts: {e}");
            }
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
