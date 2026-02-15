//! 域名元数据相关命令

use dns_orchestrator_app::AppState;
use tauri::State;

use crate::error::AppError;
use crate::types::{
    ApiResponse, BatchTagRequest, BatchTagResult, DomainMetadata, DomainMetadataUpdate,
};

/// 获取域名元数据
#[tauri::command]
pub async fn get_domain_metadata(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<DomainMetadata>, AppError> {
    let metadata = state
        .domain_metadata_service
        .get_metadata(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(metadata))
}

/// 切换收藏状态
#[tauri::command]
pub async fn toggle_domain_favorite(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<bool>, AppError> {
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
) -> Result<ApiResponse<Vec<String>>, AppError> {
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
) -> Result<ApiResponse<Vec<String>>, AppError> {
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
) -> Result<ApiResponse<Vec<String>>, AppError> {
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
) -> Result<ApiResponse<Vec<String>>, AppError> {
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
) -> Result<ApiResponse<Vec<String>>, AppError> {
    let keys = state.domain_metadata_service.find_by_tag(&tag).await?;

    // 返回 domain_id 列表（带 account_id 前缀）
    let result = keys.into_iter().map(|k| k.to_storage_key()).collect();

    Ok(ApiResponse::success(result))
}

/// 获取所有标签（用于自动补全）
#[tauri::command]
pub async fn list_all_domain_tags(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<String>>, AppError> {
    let tags = state.domain_metadata_service.list_all_tags().await?;

    Ok(ApiResponse::success(tags))
}

/// 批量添加标签
#[tauri::command]
pub async fn batch_add_domain_tags(
    state: State<'_, AppState>,
    requests: Vec<BatchTagRequest>,
) -> Result<ApiResponse<BatchTagResult>, AppError> {
    let result = state
        .domain_metadata_service
        .batch_add_tags(requests)
        .await?;

    Ok(ApiResponse::success(result))
}

/// 批量移除标签
#[tauri::command]
pub async fn batch_remove_domain_tags(
    state: State<'_, AppState>,
    requests: Vec<BatchTagRequest>,
) -> Result<ApiResponse<BatchTagResult>, AppError> {
    let result = state
        .domain_metadata_service
        .batch_remove_tags(requests)
        .await?;

    Ok(ApiResponse::success(result))
}

/// 批量替换标签
#[tauri::command]
pub async fn batch_set_domain_tags(
    state: State<'_, AppState>,
    requests: Vec<BatchTagRequest>,
) -> Result<ApiResponse<BatchTagResult>, AppError> {
    let result = state
        .domain_metadata_service
        .batch_set_tags(requests)
        .await?;

    Ok(ApiResponse::success(result))
}

/// 更新域名元数据（通用部分更新）
#[tauri::command]
pub async fn update_domain_metadata(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
    update: DomainMetadataUpdate,
) -> Result<ApiResponse<DomainMetadata>, AppError> {
    // 调用 update_metadata 方法（带验证）
    state
        .domain_metadata_service
        .update_metadata(&account_id, &domain_id, update)
        .await?;

    // 返回更新后的完整元数据
    let metadata = state
        .domain_metadata_service
        .get_metadata(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(metadata))
}
