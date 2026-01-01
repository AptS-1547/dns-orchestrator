//! 域名管理服务

use std::sync::Arc;

use dns_orchestrator_provider::ProviderError;

use crate::error::{CoreError, CoreResult};
use crate::services::{DomainMetadataService, ServiceContext};
use crate::types::{AppDomain, DomainMetadataKey, PaginatedResponse, PaginationParams};

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
    ) -> CoreResult<PaginatedResponse<AppDomain>> {
        let provider = self.ctx.get_provider(account_id).await?;

        let params = PaginationParams {
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(20),
        };

        match provider.list_domains(&params).await {
            Ok(lib_response) => {
                let mut domains: Vec<AppDomain> = lib_response
                    .items
                    .into_iter()
                    .map(|d| AppDomain::from_provider(d, account_id.to_string()))
                    .collect();

                // 批量加载元数据并合并
                let keys: Vec<(String, String)> = domains
                    .iter()
                    .map(|d| (d.account_id.clone(), d.id.clone()))
                    .collect();

                let metadata_service =
                    DomainMetadataService::new(Arc::clone(&self.ctx.domain_metadata_repository));

                if let Ok(metadata_map) = metadata_service.get_metadata_batch(keys).await {
                    for domain in &mut domains {
                        let key =
                            DomainMetadataKey::new(domain.account_id.clone(), domain.id.clone());
                        if let Some(metadata) = metadata_map.get(&key) {
                            domain.metadata = Some(metadata.clone());
                        }
                    }
                }

                Ok(PaginatedResponse::new(
                    domains,
                    lib_response.page,
                    lib_response.page_size,
                    lib_response.total_count,
                ))
            }
            Err(e) => Err(self.handle_provider_error(account_id, e).await),
        }
    }

    /// 获取域名详情
    pub async fn get_domain(&self, account_id: &str, domain_id: &str) -> CoreResult<AppDomain> {
        let provider = self.ctx.get_provider(account_id).await?;

        match provider.get_domain(domain_id).await {
            Ok(provider_domain) => Ok(AppDomain::from_provider(
                provider_domain,
                account_id.to_string(),
            )),
            Err(e) => Err(self.handle_provider_error(account_id, e).await),
        }
    }

    /// 处理 Provider 错误，如果是凭证失效则更新账户状态
    async fn handle_provider_error(&self, account_id: &str, err: ProviderError) -> CoreError {
        if let ProviderError::InvalidCredentials { .. } = &err {
            self.ctx
                .mark_account_invalid(account_id, "凭证已失效")
                .await;
        }
        CoreError::Provider(err)
    }
}
