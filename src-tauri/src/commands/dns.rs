use tauri::State;

use crate::error::DnsError;
use crate::types::{
    ApiResponse, BatchDeleteFailure, BatchDeleteRequest, BatchDeleteResult, CreateDnsRecordRequest,
    DnsRecord, PaginatedResponse, RecordQueryParams, UpdateDnsRecordRequest,
};
use crate::AppState;

/// 列出域名下的所有 DNS 记录（分页 + 搜索）
#[tauri::command]
pub async fn list_dns_records(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
    page: Option<u32>,
    page_size: Option<u32>,
    keyword: Option<String>,
    record_type: Option<String>,
) -> Result<ApiResponse<PaginatedResponse<DnsRecord>>, DnsError> {
    // 获取 provider
    let provider = state
        .registry
        .get(&account_id)
        .await
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()))?;

    // 构造查询参数
    let params = RecordQueryParams {
        page: page.unwrap_or(1),
        page_size: page_size.unwrap_or(20),
        keyword,
        record_type,
    };

    // 调用 provider 获取 DNS 记录列表
    let response = provider.list_records(&domain_id, &params).await?;

    Ok(ApiResponse::success(response))
}

/// 创建 DNS 记录
#[tauri::command]
pub async fn create_dns_record(
    state: State<'_, AppState>,
    account_id: String,
    request: CreateDnsRecordRequest,
) -> Result<ApiResponse<DnsRecord>, DnsError> {
    // 获取 provider
    let provider = state
        .registry
        .get(&account_id)
        .await
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()))?;

    // 调用 provider 创建记录
    let record = provider.create_record(&request).await?;

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
    // 获取 provider
    let provider = state
        .registry
        .get(&account_id)
        .await
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()))?;

    // 调用 provider 更新记录
    let record = provider.update_record(&record_id, &request).await?;

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
    // 获取 provider
    let provider = state
        .registry
        .get(&account_id)
        .await
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()))?;

    // 调用 provider 删除记录
    provider.delete_record(&record_id, &domain_id).await?;

    Ok(ApiResponse::success(()))
}

/// 批量删除 DNS 记录
#[tauri::command]
pub async fn batch_delete_dns_records(
    state: State<'_, AppState>,
    account_id: String,
    request: BatchDeleteRequest,
) -> Result<ApiResponse<BatchDeleteResult>, DnsError> {
    // 获取 provider
    let provider = state
        .registry
        .get(&account_id)
        .await
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()))?;

    let mut success_count = 0;
    let mut failures = Vec::new();

    // 并行删除所有记录
    let delete_futures: Vec<_> = request
        .record_ids
        .iter()
        .map(|record_id| {
            let provider = provider.clone();
            let domain_id = request.domain_id.clone();
            let record_id = record_id.clone();
            async move {
                match provider.delete_record(&record_id, &domain_id).await {
                    Ok(()) => Ok(record_id),
                    Err(e) => Err((record_id, e.to_string())),
                }
            }
        })
        .collect();

    let results = futures::future::join_all(delete_futures).await;

    for result in results {
        match result {
            Ok(_) => success_count += 1,
            Err((record_id, reason)) => {
                failures.push(BatchDeleteFailure { record_id, reason });
            }
        }
    }

    Ok(ApiResponse::success(BatchDeleteResult {
        success_count,
        failed_count: failures.len(),
        failures,
    }))
}
