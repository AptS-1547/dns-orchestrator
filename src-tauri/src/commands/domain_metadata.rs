//! 域名元数据相关命令

use chrono::{DateTime, Utc};
use tauri::State;

use crate::error::DnsError;
use crate::types::ApiResponse;
use crate::AppState;

use serde::{Deserialize, Serialize};

// 本地类型定义（与前端对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadata {
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub color: String,
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorited_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

// 类型转换
impl From<dns_orchestrator_core::types::DomainMetadata> for DomainMetadata {
    fn from(core: dns_orchestrator_core::types::DomainMetadata) -> Self {
        Self {
            is_favorite: core.is_favorite,
            tags: core.tags,
            color: core.color,
            note: core.note,
            favorited_at: core.favorited_at,
            updated_at: core.updated_at,
        }
    }
}

/// 获取域名元数据
#[tauri::command]
pub async fn get_domain_metadata(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<DomainMetadata>, DnsError> {
    let metadata = state
        .domain_metadata_service
        .get_metadata(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(metadata.into()))
}

/// 切换收藏状态
#[tauri::command]
pub async fn toggle_domain_favorite(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<bool>, DnsError> {
    let new_state = state
        .domain_metadata_service
        .toggle_favorite(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(new_state))
}

/// 获取账户下的收藏域名 ID 列表
#[tauri::command]
pub async fn list_account_favorite_domain_keys(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<ApiResponse<Vec<String>>, DnsError> {
    let keys = state
        .domain_metadata_service
        .list_favorites(&account_id)
        .await?;

    let result = keys.into_iter().map(|k| k.domain_id).collect();

    Ok(ApiResponse::success(result))
}

/// 添加标签
#[tauri::command]
pub async fn add_domain_tag(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
    tag: String,
) -> Result<ApiResponse<Vec<String>>, DnsError> {
    let tags = state
        .domain_metadata_service
        .add_tag(&account_id, &domain_id, tag)
        .await?;

    Ok(ApiResponse::success(tags))
}

/// 移除标签
#[tauri::command]
pub async fn remove_domain_tag(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
    tag: String,
) -> Result<ApiResponse<Vec<String>>, DnsError> {
    let tags = state
        .domain_metadata_service
        .remove_tag(&account_id, &domain_id, &tag)
        .await?;

    Ok(ApiResponse::success(tags))
}

/// 批量设置标签
#[tauri::command]
pub async fn set_domain_tags(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
    tags: Vec<String>,
) -> Result<ApiResponse<Vec<String>>, DnsError> {
    let tags = state
        .domain_metadata_service
        .set_tags(&account_id, &domain_id, tags)
        .await?;

    Ok(ApiResponse::success(tags))
}

/// 按标签查询域名
#[tauri::command]
pub async fn find_domains_by_tag(
    state: State<'_, AppState>,
    tag: String,
) -> Result<ApiResponse<Vec<String>>, DnsError> {
    let keys = state.domain_metadata_service.find_by_tag(&tag).await?;

    // 返回 domain_id 列表（带 account_id 前缀）
    let result = keys.into_iter().map(|k| k.to_storage_key()).collect();

    Ok(ApiResponse::success(result))
}

/// 获取所有标签（用于自动补全）
#[tauri::command]
pub async fn list_all_domain_tags(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<String>>, DnsError> {
    let tags = state.domain_metadata_service.list_all_tags().await?;

    Ok(ApiResponse::success(tags))
}

// ===== 批量标签操作 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagRequest {
    pub account_id: String,
    pub domain_id: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failures: Vec<BatchTagFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagFailure {
    pub account_id: String,
    pub domain_id: String,
    pub reason: String,
}

// 类型转换
impl From<dns_orchestrator_core::types::BatchTagRequest> for BatchTagRequest {
    fn from(core: dns_orchestrator_core::types::BatchTagRequest) -> Self {
        Self {
            account_id: core.account_id,
            domain_id: core.domain_id,
            tags: core.tags,
        }
    }
}

impl From<BatchTagRequest> for dns_orchestrator_core::types::BatchTagRequest {
    fn from(local: BatchTagRequest) -> Self {
        Self {
            account_id: local.account_id,
            domain_id: local.domain_id,
            tags: local.tags,
        }
    }
}

impl From<dns_orchestrator_core::types::BatchTagResult> for BatchTagResult {
    fn from(core: dns_orchestrator_core::types::BatchTagResult) -> Self {
        Self {
            success_count: core.success_count,
            failed_count: core.failed_count,
            failures: core
                .failures
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        }
    }
}

impl From<dns_orchestrator_core::types::BatchTagFailure> for BatchTagFailure {
    fn from(core: dns_orchestrator_core::types::BatchTagFailure) -> Self {
        Self {
            account_id: core.account_id,
            domain_id: core.domain_id,
            reason: core.reason,
        }
    }
}

/// 批量添加标签
#[tauri::command]
pub async fn batch_add_domain_tags(
    state: State<'_, AppState>,
    requests: Vec<BatchTagRequest>,
) -> Result<ApiResponse<BatchTagResult>, DnsError> {
    let core_requests: Vec<dns_orchestrator_core::types::BatchTagRequest> =
        requests.into_iter().map(std::convert::Into::into).collect();

    let result = state
        .domain_metadata_service
        .batch_add_tags(core_requests)
        .await?;

    Ok(ApiResponse::success(result.into()))
}

/// 批量移除标签
#[tauri::command]
pub async fn batch_remove_domain_tags(
    state: State<'_, AppState>,
    requests: Vec<BatchTagRequest>,
) -> Result<ApiResponse<BatchTagResult>, DnsError> {
    let core_requests: Vec<dns_orchestrator_core::types::BatchTagRequest> =
        requests.into_iter().map(std::convert::Into::into).collect();

    let result = state
        .domain_metadata_service
        .batch_remove_tags(core_requests)
        .await?;

    Ok(ApiResponse::success(result.into()))
}

/// 批量替换标签
#[tauri::command]
pub async fn batch_set_domain_tags(
    state: State<'_, AppState>,
    requests: Vec<BatchTagRequest>,
) -> Result<ApiResponse<BatchTagResult>, DnsError> {
    let core_requests: Vec<dns_orchestrator_core::types::BatchTagRequest> =
        requests.into_iter().map(std::convert::Into::into).collect();

    let result = state
        .domain_metadata_service
        .batch_set_tags(core_requests)
        .await?;

    Ok(ApiResponse::success(result.into()))
}

// ===== 通用元数据更新（Phase 3） =====

/// 元数据部分更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadataUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Option<String>>,
}

// 类型转换
impl From<DomainMetadataUpdate> for dns_orchestrator_core::types::DomainMetadataUpdate {
    fn from(local: DomainMetadataUpdate) -> Self {
        Self {
            is_favorite: local.is_favorite,
            tags: local.tags,
            color: local.color,
            note: local.note,
        }
    }
}

/// 更新域名元数据（通用部分更新）
#[tauri::command]
pub async fn update_domain_metadata(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
    update: DomainMetadataUpdate,
) -> Result<ApiResponse<DomainMetadata>, DnsError> {
    // 调用 update_metadata 方法（带验证）
    state
        .domain_metadata_service
        .update_metadata(&account_id, &domain_id, update.into())
        .await?;

    // 返回更新后的完整元数据
    let metadata = state
        .domain_metadata_service
        .get_metadata(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(metadata.into()))
}
