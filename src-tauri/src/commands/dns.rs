use tauri::State;

use crate::error::DnsError;
use crate::types::{
    ApiResponse, BatchDeleteRequest, BatchDeleteResult, CreateDnsRecordRequest, DnsRecord,
    DnsRecordType, PaginatedResponse, UpdateDnsRecordRequest,
};
use crate::AppState;

// 从 core 类型转换到本地类型的辅助函数
fn convert_batch_delete_result(
    result: dns_orchestrator_core::types::BatchDeleteResult,
) -> BatchDeleteResult {
    BatchDeleteResult {
        success_count: result.success_count,
        failed_count: result.failed_count,
        failures: result
            .failures
            .into_iter()
            .map(|f| crate::types::BatchDeleteFailure {
                record_id: f.record_id,
                reason: f.reason,
            })
            .collect(),
    }
}

/// 列出域名下的所有 DNS 记录（分页 + 搜索）
#[tauri::command]
pub async fn list_dns_records(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
    page: Option<u32>,
    page_size: Option<u32>,
    keyword: Option<String>,
    record_type: Option<DnsRecordType>,
) -> Result<ApiResponse<PaginatedResponse<DnsRecord>>, DnsError> {
    let response = state
        .dns_service
        .list_records(
            &account_id,
            &domain_id,
            page,
            page_size,
            keyword,
            record_type,
        )
        .await?;

    Ok(ApiResponse::success(response))
}

/// 创建 DNS 记录
#[tauri::command]
pub async fn create_dns_record(
    state: State<'_, AppState>,
    account_id: String,
    request: CreateDnsRecordRequest,
) -> Result<ApiResponse<DnsRecord>, DnsError> {
    let record = state
        .dns_service
        .create_record(&account_id, request)
        .await?;

    Ok(ApiResponse::success(record))
}

/// 更新 DNS 记录
#[tauri::command]
pub async fn update_dns_record(
    state: State<'_, AppState>,
    account_id: String,
    record_id: String,
    request: UpdateDnsRecordRequest,
) -> Result<ApiResponse<DnsRecord>, DnsError> {
    let record = state
        .dns_service
        .update_record(&account_id, &record_id, request)
        .await?;

    Ok(ApiResponse::success(record))
}

/// 删除 DNS 记录
#[tauri::command]
pub async fn delete_dns_record(
    state: State<'_, AppState>,
    account_id: String,
    record_id: String,
    domain_id: String,
) -> Result<ApiResponse<()>, DnsError> {
    state
        .dns_service
        .delete_record(&account_id, &record_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(()))
}

/// 批量删除 DNS 记录
#[tauri::command]
pub async fn batch_delete_dns_records(
    state: State<'_, AppState>,
    account_id: String,
    request: BatchDeleteRequest,
) -> Result<ApiResponse<BatchDeleteResult>, DnsError> {
    // 转换请求类型
    let core_request = dns_orchestrator_core::types::BatchDeleteRequest {
        domain_id: request.domain_id,
        record_ids: request.record_ids,
    };

    let result = state
        .dns_service
        .batch_delete_records(&account_id, core_request)
        .await?;

    Ok(ApiResponse::success(convert_batch_delete_result(result)))
}
