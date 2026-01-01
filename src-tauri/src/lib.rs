mod adapters;
mod commands;
mod error;
mod types;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(target_os = "android")]
use commands::updater;
use commands::{account, dns, domain, toolbox};
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

use adapters::{TauriAccountRepository, TauriCredentialStore};
use dns_orchestrator_core::services::{
    AccountBootstrapService, AccountLifecycleService, AccountMetadataService,
    CredentialManagementService, DnsService, DomainService, ImportExportService,
    ProviderMetadataService, ServiceContext,
};
use dns_orchestrator_core::traits::InMemoryProviderRegistry;

/// 应用全局状态
pub struct AppState {
    /// 服务上下文
    pub ctx: Arc<ServiceContext>,
    /// 账户元数据服务
    pub account_metadata_service: Arc<AccountMetadataService>,
    /// 凭证管理服务
    pub credential_management_service: Arc<CredentialManagementService>,
    /// 账户生命周期服务
    pub account_lifecycle_service: Arc<AccountLifecycleService>,
    /// 账户启动恢复服务
    pub account_bootstrap_service: Arc<AccountBootstrapService>,
    /// Provider 元数据服务
    pub provider_metadata_service: ProviderMetadataService,
    /// 导入导出服务
    pub import_export_service: ImportExportService,
    /// 域名服务
    pub domain_service: DomainService,
    /// DNS 服务
    pub dns_service: DnsService,
    /// 账户恢复是否完成
    pub restore_completed: AtomicBool,
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        // 创建适配器（Android 版本需要 AppHandle）
        #[cfg(not(target_os = "android"))]
        let credential_store = Arc::new(TauriCredentialStore::new());

        #[cfg(target_os = "android")]
        let credential_store = Arc::new(TauriCredentialStore::new(app_handle.clone()));

        let account_repository = Arc::new(TauriAccountRepository::new(app_handle));
        let provider_registry = Arc::new(InMemoryProviderRegistry::new());

        // 创建服务上下文
        let ctx = Arc::new(ServiceContext::new(
            credential_store.clone(),
            account_repository.clone(),
            provider_registry.clone(),
        ));

        // 创建细粒度账户服务
        let account_metadata_service = Arc::new(AccountMetadataService::new(account_repository));
        let credential_management_service = Arc::new(CredentialManagementService::new(
            credential_store,
            provider_registry,
        ));
        let account_lifecycle_service = Arc::new(AccountLifecycleService::new(
            Arc::clone(&account_metadata_service),
            Arc::clone(&credential_management_service),
        ));
        let account_bootstrap_service = Arc::new(AccountBootstrapService::new(
            Arc::clone(&account_metadata_service),
            Arc::clone(&credential_management_service),
        ));
        let provider_metadata_service = ProviderMetadataService::new();

        // 创建其他服务
        let import_export_service = ImportExportService::new(Arc::clone(&ctx));
        let domain_service = DomainService::new(Arc::clone(&ctx));
        let dns_service = DnsService::new(Arc::clone(&ctx));

        Self {
            ctx,
            account_metadata_service,
            credential_management_service,
            account_lifecycle_service,
            account_bootstrap_service,
            provider_metadata_service,
            import_export_service,
            domain_service,
            dns_service,
            restore_completed: AtomicBool::new(false),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(
            tauri_plugin_log::Builder::new()
                .targets([Target::new(TargetKind::Stdout)])
                .level(log::LevelFilter::Debug)
                .build(),
        );
    }

    #[cfg(not(debug_assertions))]
    {
        builder = builder.plugin(
            tauri_plugin_log::Builder::new()
                .targets([Target::new(TargetKind::Stdout)])
                .level(log::LevelFilter::Warn)
                .build(),
        );
    }

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
        app.manage(state);

        // 后台恢复账户，不阻塞启动
        let app_handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let state = app_handle.state::<AppState>();
            let result = state.account_bootstrap_service.restore_accounts().await;

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

            state.restore_completed.store(true, Ordering::SeqCst);
        });

        Ok(())
    });

    #[cfg(not(target_os = "android"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        // Account commands
        account::list_accounts,
        account::create_account,
        account::update_account,
        account::delete_account,
        account::batch_delete_accounts,
        account::list_providers,
        account::export_accounts,
        account::preview_import,
        account::import_accounts,
        account::is_restore_completed,
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
        toolbox::http_header_check,
        toolbox::dns_propagation_check,
        toolbox::dnssec_check,
    ]);

    #[cfg(target_os = "android")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        // Account commands
        account::list_accounts,
        account::create_account,
        account::update_account,
        account::delete_account,
        account::batch_delete_accounts,
        account::list_providers,
        account::export_accounts,
        account::preview_import,
        account::import_accounts,
        account::is_restore_completed,
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
        toolbox::http_header_check,
        toolbox::dns_propagation_check,
        toolbox::dnssec_check,
        // Android updater commands
        updater::check_android_update,
        updater::download_apk,
        updater::install_apk,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
