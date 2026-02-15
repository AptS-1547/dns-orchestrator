//! Domain management service.

use std::sync::Arc;

use crate::error::CoreResult;
use crate::services::{DomainMetadataService, ServiceContext};
use crate::types::{AppDomain, DomainMetadataKey, PaginatedResponse, PaginationParams};

/// Service for domain listing and lookup.
pub struct DomainService {
    ctx: Arc<ServiceContext>,
    metadata_service: Arc<DomainMetadataService>,
}

impl DomainService {
    /// Creates a domain service.
    #[must_use]
    pub fn new(ctx: Arc<ServiceContext>, metadata_service: Arc<DomainMetadataService>) -> Self {
        Self {
            ctx,
            metadata_service,
        }
    }

    /// Lists all domains under an account (paginated).
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

                // Batch-load metadata and merge into each domain item.
                let keys: Vec<(String, String)> = domains
                    .iter()
                    .map(|d| (d.account_id.clone(), d.id.clone()))
                    .collect();

                if let Ok(metadata_map) = self.metadata_service.get_metadata_batch(keys).await {
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
            Err(e) => Err(self.ctx.handle_provider_error(account_id, e).await),
        }
    }

    /// Gets a single domain by ID.
    pub async fn get_domain(&self, account_id: &str, domain_id: &str) -> CoreResult<AppDomain> {
        let provider = self.ctx.get_provider(account_id).await?;

        match provider.get_domain(domain_id).await {
            Ok(provider_domain) => Ok(AppDomain::from_provider(
                provider_domain,
                account_id.to_string(),
            )),
            Err(e) => Err(self.ctx.handle_provider_error(account_id, e).await),
        }
    }
}
