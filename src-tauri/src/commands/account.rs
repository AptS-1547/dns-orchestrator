use tauri::State;

use crate::error::DnsError;
use crate::types::{
    Account, ApiResponse, CreateAccountRequest, ExportAccountsRequest, ExportAccountsResponse,
    ImportAccountsRequest, ImportPreview, ImportResult, ProviderMetadata,
};
use crate::AppState;

// 从 core 类型转换到本地类型的辅助函数
fn convert_account(core_account: dns_orchestrator_core::types::Account) -> Account {
    Account {
        id: core_account.id,
        name: core_account.name,
        provider: core_account.provider,
        created_at: core_account.created_at,
        updated_at: core_account.updated_at,
        status: core_account.status.map(convert_account_status),
        error: core_account.error,
    }
}

fn convert_account_status(
    status: dns_orchestrator_core::types::AccountStatus,
) -> crate::types::AccountStatus {
    match status {
        dns_orchestrator_core::types::AccountStatus::Active => crate::types::AccountStatus::Active,
        dns_orchestrator_core::types::AccountStatus::Error => crate::types::AccountStatus::Error,
    }
}

fn convert_export_response(
    response: dns_orchestrator_core::types::ExportAccountsResponse,
) -> ExportAccountsResponse {
    ExportAccountsResponse {
        content: response.content,
        suggested_filename: response.suggested_filename,
    }
}

fn convert_import_preview(preview: dns_orchestrator_core::types::ImportPreview) -> ImportPreview {
    ImportPreview {
        encrypted: preview.encrypted,
        account_count: preview.account_count,
        accounts: preview.accounts.map(|accounts| {
            accounts
                .into_iter()
                .map(|a| crate::types::ImportPreviewAccount {
                    name: a.name,
                    provider: a.provider,
                    has_conflict: a.has_conflict,
                })
                .collect()
        }),
    }
}

fn convert_import_result(result: dns_orchestrator_core::types::ImportResult) -> ImportResult {
    ImportResult {
        success_count: result.success_count,
        failures: result
            .failures
            .into_iter()
            .map(|f| crate::types::ImportFailure {
                name: f.name,
                reason: f.reason,
            })
            .collect(),
    }
}

/// 列出所有账号
#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Account>>, DnsError> {
    let accounts = state.account_service.list_accounts().await?;
    let converted: Vec<Account> = accounts.into_iter().map(convert_account).collect();
    Ok(ApiResponse::success(converted))
}

/// 创建新账号
#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    request: CreateAccountRequest,
) -> Result<ApiResponse<Account>, DnsError> {
    // 转换请求类型
    let core_request = dns_orchestrator_core::types::CreateAccountRequest {
        name: request.name,
        provider: request.provider,
        credentials: request.credentials,
    };

    let account = state.account_service.create_account(core_request).await?;
    Ok(ApiResponse::success(convert_account(account)))
}

/// 删除账号
#[tauri::command]
pub async fn delete_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<ApiResponse<()>, DnsError> {
    state.account_service.delete_account(&account_id).await?;
    Ok(ApiResponse::success(()))
}

/// 获取所有支持的提供商列表
#[tauri::command]
pub async fn list_providers(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ProviderMetadata>>, DnsError> {
    let providers = state.account_service.list_providers();
    Ok(ApiResponse::success(providers))
}

/// 导出账号
#[tauri::command]
pub async fn export_accounts(
    state: State<'_, AppState>,
    request: ExportAccountsRequest,
) -> Result<ApiResponse<ExportAccountsResponse>, DnsError> {
    let core_request = dns_orchestrator_core::types::ExportAccountsRequest {
        account_ids: request.account_ids,
        encrypt: request.encrypt,
        password: request.password,
    };

    let app_version = env!("CARGO_PKG_VERSION");
    let response = state
        .account_service
        .export_accounts(core_request, app_version)
        .await?;

    Ok(ApiResponse::success(convert_export_response(response)))
}

/// 预览导入文件
#[tauri::command]
pub async fn preview_import(
    state: State<'_, AppState>,
    content: String,
    password: Option<String>,
) -> Result<ApiResponse<ImportPreview>, DnsError> {
    let preview = state
        .account_service
        .preview_import(&content, password.as_deref())
        .await?;

    Ok(ApiResponse::success(convert_import_preview(preview)))
}

/// 执行导入
#[tauri::command]
pub async fn import_accounts(
    state: State<'_, AppState>,
    request: ImportAccountsRequest,
) -> Result<ApiResponse<ImportResult>, DnsError> {
    let core_request = dns_orchestrator_core::types::ImportAccountsRequest {
        content: request.content,
        password: request.password,
    };

    let result = state.account_service.import_accounts(core_request).await?;

    Ok(ApiResponse::success(convert_import_result(result)))
}
