use dns_orchestrator_app::AppState;
use tauri::State;

use crate::error::AppError;
use crate::types::{
    Account, ApiResponse, BatchDeleteResult, CreateAccountRequest, ExportAccountsRequest,
    ExportAccountsResponse, ImportAccountsRequest, ImportPreview, ImportResult, ProviderMetadata,
    UpdateAccountRequest,
};

/// 列出所有账号
#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Account>>, AppError> {
    let accounts = state.account_service.list_accounts().await?;
    Ok(ApiResponse::success(accounts))
}

/// 创建新账号
#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    request: CreateAccountRequest,
) -> Result<ApiResponse<Account>, AppError> {
    let account = state.account_service.create_account(request).await?;
    Ok(ApiResponse::success(account))
}

/// 删除账号
#[tauri::command]
pub async fn delete_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<ApiResponse<()>, AppError> {
    state.account_service.delete_account(&account_id).await?;
    Ok(ApiResponse::success(()))
}

/// 更新账号
#[tauri::command]
pub async fn update_account(
    state: State<'_, AppState>,
    request: UpdateAccountRequest,
) -> Result<ApiResponse<Account>, AppError> {
    let account = state.account_service.update_account(request).await?;
    Ok(ApiResponse::success(account))
}

/// 批量删除账号
#[tauri::command]
pub async fn batch_delete_accounts(
    state: State<'_, AppState>,
    account_ids: Vec<String>,
) -> Result<ApiResponse<BatchDeleteResult>, AppError> {
    let result = state
        .account_service
        .batch_delete_accounts(account_ids)
        .await?;
    Ok(ApiResponse::success(result))
}

/// 获取所有支持的提供商列表
#[tauri::command]
pub async fn list_providers(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ProviderMetadata>>, AppError> {
    let providers = state.provider_metadata_service.list_providers();
    Ok(ApiResponse::success(providers))
}

/// 导出账号
#[tauri::command]
pub async fn export_accounts(
    state: State<'_, AppState>,
    request: ExportAccountsRequest,
) -> Result<ApiResponse<ExportAccountsResponse>, AppError> {
    let app_version = env!("CARGO_PKG_VERSION");
    let response = state
        .import_export_service
        .export_accounts(request, app_version)
        .await?;

    Ok(ApiResponse::success(response))
}

/// 预览导入文件
#[tauri::command]
pub async fn preview_import(
    state: State<'_, AppState>,
    content: String,
    password: Option<String>,
) -> Result<ApiResponse<ImportPreview>, AppError> {
    let preview = state
        .import_export_service
        .preview_import(&content, password.as_deref())
        .await?;

    Ok(ApiResponse::success(preview))
}

/// 执行导入
#[tauri::command]
pub async fn import_accounts(
    state: State<'_, AppState>,
    request: ImportAccountsRequest,
) -> Result<ApiResponse<ImportResult>, AppError> {
    let result = state.import_export_service.import_accounts(request).await?;

    Ok(ApiResponse::success(result))
}

/// 检查账户恢复是否完成
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn is_restore_completed(state: State<'_, AppState>) -> bool {
    state
        .restore_completed
        .load(std::sync::atomic::Ordering::SeqCst)
}
