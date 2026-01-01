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
    CredentialManagementService, DnsService, DomainService, ImportExportService, MigrationResult,
    MigrationService, ProviderMetadataService, ServiceContext,
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

        // 执行凭证迁移（v1.7.0 - 阻塞操作，确保迁移完成后再恢复账户）
        let app_handle = app.handle().clone();
        tauri::async_runtime::block_on(async move {
            let state = app_handle.state::<AppState>();

            // 1. 备份凭证（迁移前）
            let backup_result = async {
                let raw_json = state.ctx.credential_store.load_raw_json().await?;

                let data_dir = app_handle.path().app_data_dir().map_err(|e| {
                    dns_orchestrator_core::error::CoreError::StorageError(format!(
                        "Failed to get data dir: {e}"
                    ))
                })?;

                std::fs::create_dir_all(&data_dir).map_err(|e| {
                    dns_orchestrator_core::error::CoreError::StorageError(format!(
                        "Failed to create data dir: {e}"
                    ))
                })?;

                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let backup_path = data_dir.join(format!("credentials.backup.{timestamp}.json"));

                std::fs::write(&backup_path, raw_json.as_bytes()).map_err(|e| {
                    dns_orchestrator_core::error::CoreError::StorageError(format!(
                        "Failed to write backup: {e}"
                    ))
                })?;

                log::info!("凭证已备份到: {}", backup_path.display());
                Ok::<_, dns_orchestrator_core::error::CoreError>(backup_path)
            }
            .await;

            // 保存备份路径（用于后续清理）
            let backup_path_opt = match &backup_result {
                Ok(path) => {
                    log::info!("备份成功: {}", path.display());
                    Some(path.clone())
                }
                Err(e) => {
                    log::warn!("备份失败（继续迁移）: {e}");
                    None
                }
            };

            // 2. 创建迁移服务
            let migration_service = MigrationService::new(
                Arc::clone(&state.ctx.credential_store),
                Arc::clone(&state.ctx.account_repository),
            );

            // 3. 执行迁移
            match migration_service.migrate_if_needed().await {
                Ok(MigrationResult::NotNeeded) => {
                    log::info!("凭证格式检查：无需迁移");
                    // 删除备份文件（无需迁移）
                    if let Some(backup_path) = &backup_path_opt {
                        if let Err(e) = std::fs::remove_file(backup_path) {
                            log::warn!("删除备份文件失败: {e}");
                        } else {
                            log::info!("已删除备份文件");
                        }
                    }
                }
                Ok(MigrationResult::Success {
                    migrated_count,
                    failed_accounts,
                }) => {
                    log::info!("凭证迁移成功：{migrated_count} 个账户已迁移");
                    if !failed_accounts.is_empty() {
                        log::warn!(
                            "部分账户迁移失败 ({} 个): {:?}",
                            failed_accounts.len(),
                            failed_accounts
                        );

                        // 将失败的账户标记为 Error 状态
                        for (account_id, error_msg) in &failed_accounts {
                            if let Err(e) = state
                                .account_metadata_service
                                .update_status(
                                    account_id,
                                    dns_orchestrator_core::types::AccountStatus::Error,
                                    Some(format!("凭证迁移失败: {error_msg}")),
                                )
                                .await
                            {
                                log::error!("更新账户 {account_id} 状态失败: {e}");
                            }
                        }
                    }

                    // 删除备份文件（迁移成功）
                    if let Some(backup_path) = &backup_path_opt {
                        if let Err(e) = std::fs::remove_file(backup_path) {
                            log::warn!("删除备份文件失败: {e}");
                        } else {
                            log::info!("已删除备份文件（迁移成功）");
                        }
                    }
                }
                Err(e) => {
                    log::error!("凭证迁移失败: {e}");
                    // 保留备份文件供手动恢复
                    if let Some(backup_path) = &backup_path_opt {
                        log::error!(
                            "迁移失败，备份文件保留在: {}，请手动检查",
                            backup_path.display()
                        );
                    }
                    // 不中断启动，继续尝试恢复（可能会因为格式问题导致部分账户恢复失败）
                }
            }
        });

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
