use tauri::State;

use crate::error::{DnsError, ProviderError};
use crate::types::{AccountStatus, ApiResponse, Domain, PaginatedResponse, PaginationParams};
use crate::AppState;

/// 更新账户状态（凭证失效时调用）
async fn mark_account_invalid(state: &AppState, account_id: &str, error_msg: &str) {
    let mut accounts = state.accounts.write().await;
    if let Some(account) = accounts.iter_mut().find(|a| a.id == account_id) {
        account.status = Some(AccountStatus::Error);
        account.error = Some(error_msg.to_string());
        log::warn!("Account {account_id} marked as invalid: {error_msg}");
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
    // 获取 provider
    let provider = state
        .registry
        .get(&account_id)
        .await
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()))?;

    // 构造分页参数
    let params = PaginationParams {
        page: page.unwrap_or(1),
        page_size: page_size.unwrap_or(20),
    };

    // 调用 provider 获取域名列表
    match provider.list_domains(&params).await {
        Ok(response) => Ok(ApiResponse::success(response)),
        Err(DnsError::Provider(ProviderError::InvalidCredentials { .. })) => {
            // 凭证失效，更新账户状态
            mark_account_invalid(&state, &account_id, "凭证已失效").await;
            Err(DnsError::Provider(ProviderError::InvalidCredentials {
                provider: "unknown".to_string(),
            }))
        }
        Err(e) => Err(e),
    }
}

/// 获取域名详情
#[tauri::command]
pub async fn get_domain(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<Domain>, DnsError> {
    // 获取 provider
    let provider = state
        .registry
        .get(&account_id)
        .await
        .ok_or_else(|| DnsError::AccountNotFound(account_id.clone()))?;

    // 调用 provider 获取域名详情
    let domain = provider.get_domain(&domain_id).await?;

    Ok(ApiResponse::success(domain))
}
