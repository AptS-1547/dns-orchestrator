//! 域名元数据相关命令

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
    pub color: Option<String>,
    pub note: Option<String>,
    pub updated_at: i64,
}

// 类型转换
impl From<dns_orchestrator_core::types::DomainMetadata> for DomainMetadata {
    fn from(core: dns_orchestrator_core::types::DomainMetadata) -> Self {
        Self {
            is_favorite: core.is_favorite,
            tags: core.tags,
            color: core.color,
            note: core.note,
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
