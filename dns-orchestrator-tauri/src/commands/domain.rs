use dns_orchestrator_app::AppState;
use tauri::State;

use crate::error::AppError;
use crate::types::{ApiResponse, Domain, PaginatedResponse};

/// 列出账号下的所有域名（分页）
#[tauri::command]
pub async fn list_domains(
    state: State<'_, AppState>,
    account_id: String,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<ApiResponse<PaginatedResponse<Domain>>, AppError> {
    let response = state
        .domain_service
        .list_domains(&account_id, page, page_size)
        .await?;

    Ok(ApiResponse::success(response))
}

/// 获取域名详情
#[tauri::command]
pub async fn get_domain(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<Domain>, AppError> {
    let domain = state
        .domain_service
        .get_domain(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(domain))
}
