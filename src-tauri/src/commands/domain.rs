use tauri::State;

use crate::error::DnsError;
use crate::types::{ApiResponse, Domain, PaginatedResponse};
use crate::AppState;

// 从 core 类型转换到本地类型的辅助函数
fn convert_domain(core_domain: dns_orchestrator_core::types::AppDomain) -> Domain {
    Domain {
        id: core_domain.id,
        name: core_domain.name,
        account_id: core_domain.account_id,
        provider: core_domain.provider,
        status: core_domain.status,
        record_count: core_domain.record_count,
        metadata: core_domain.metadata,
    }
}

/// 列出账号下的所有域名（分页）
#[tauri::command]
pub async fn list_domains(
    state: State<'_, AppState>,
    account_id: String,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<ApiResponse<PaginatedResponse<Domain>>, DnsError> {
    let response = state
        .domain_service
        .list_domains(&account_id, page, page_size)
        .await?;

    // 转换响应中的 Domain 类型
    let converted_items: Vec<Domain> = response.items.into_iter().map(convert_domain).collect();

    let result = PaginatedResponse::new(
        converted_items,
        response.page,
        response.page_size,
        response.total_count,
    );

    Ok(ApiResponse::success(result))
}

/// 获取域名详情
#[tauri::command]
pub async fn get_domain(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<Domain>, DnsError> {
    let domain = state
        .domain_service
        .get_domain(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(convert_domain(domain)))
}
