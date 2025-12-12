//! 域名管理服务

use std::sync::Arc;

use dns_orchestrator_provider::{DnsProvider, ProviderError};

use crate::error::{CoreError, CoreResult};
use crate::services::ServiceContext;
use crate::types::{AccountStatus, Domain, PaginatedResponse, PaginationParams};

/// 域名管理服务
pub struct DomainService {
    ctx: Arc<ServiceContext>,
}

impl DomainService {
    /// 创建域名服务实例
    #[must_use]
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// 列出账号下的所有域名（分页）
    pub async fn list_domains(
        &self,
        account_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> CoreResult<PaginatedResponse<Domain>> {
        let provider = self.get_provider(account_id).await?;

        let params = PaginationParams {
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(20),
        };

        match provider.list_domains(&params).await {
            Ok(lib_response) => {
                let domains: Vec<Domain> = lib_response
                    .items
                    .into_iter()
                    .map(|d| Domain::from_lib(d, account_id.to_string()))
                    .collect();

                Ok(PaginatedResponse::new(
                    domains,
                    lib_response.page,
                    lib_response.page_size,
                    lib_response.total_count,
                ))
            }
            Err(ProviderError::InvalidCredentials { provider, .. }) => {
                // 凭证失效，更新账户状态
                self.mark_account_invalid(account_id, "凭证已失效").await;
                Err(CoreError::Provider(ProviderError::InvalidCredentials {
                    provider,
                    raw_message: None,
                }))
            }
            Err(e) => Err(CoreError::Provider(e)),
        }
    }

    /// 获取域名详情
    pub async fn get_domain(&self, account_id: &str, domain_id: &str) -> CoreResult<Domain> {
        let provider = self.get_provider(account_id).await?;

        let lib_domain = provider.get_domain(domain_id).await?;
        Ok(Domain::from_lib(lib_domain, account_id.to_string()))
    }

    /// 获取 Provider 实例
    async fn get_provider(&self, account_id: &str) -> CoreResult<Arc<dyn DnsProvider>> {
        self.ctx
            .provider_registry
            .get(account_id)
            .await
            .ok_or_else(|| CoreError::AccountNotFound(account_id.to_string()))
    }

    /// 标记账户为无效状态
    async fn mark_account_invalid(&self, account_id: &str, error_msg: &str) {
        let _ = self
            .ctx
            .account_repository
            .update_status(
                account_id,
                AccountStatus::Error,
                Some(error_msg.to_string()),
            )
            .await;
        log::warn!("Account {account_id} marked as invalid: {error_msg}");
    }
}
