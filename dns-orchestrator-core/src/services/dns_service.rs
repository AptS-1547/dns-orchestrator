//! DNS record management service

use std::sync::Arc;

use dns_orchestrator_provider::ProviderError;
use futures::stream::{self, StreamExt};

use crate::error::CoreResult;
use crate::services::ServiceContext;
use crate::types::{
    BatchDeleteFailure, BatchDeleteRequest, BatchDeleteResult, CreateDnsRecordRequest, DnsRecord,
    DnsRecordType, PaginatedResponse, RecordQueryParams, UpdateDnsRecordRequest,
};

/// Service for DNS record operations.
pub struct DnsService {
    ctx: Arc<ServiceContext>,
}

impl DnsService {
    /// Creates a DNS service.
    #[must_use]
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// Lists DNS records for a domain with pagination and filters.
    pub async fn list_records(
        &self,
        account_id: &str,
        domain_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        keyword: Option<String>,
        record_type: Option<DnsRecordType>,
    ) -> CoreResult<PaginatedResponse<DnsRecord>> {
        let provider = self.ctx.get_provider(account_id).await?;

        let params = RecordQueryParams {
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(20),
            keyword,
            record_type,
        };

        match provider.list_records(domain_id, &params).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.ctx.handle_provider_error(account_id, e).await),
        }
    }

    /// Creates one DNS record.
    pub async fn create_record(
        &self,
        account_id: &str,
        request: CreateDnsRecordRequest,
    ) -> CoreResult<DnsRecord> {
        let provider = self.ctx.get_provider(account_id).await?;
        match provider.create_record(&request).await {
            Ok(record) => Ok(record),
            Err(e) => Err(self.ctx.handle_provider_error(account_id, e).await),
        }
    }

    /// Updates one DNS record.
    pub async fn update_record(
        &self,
        account_id: &str,
        record_id: &str,
        request: UpdateDnsRecordRequest,
    ) -> CoreResult<DnsRecord> {
        let provider = self.ctx.get_provider(account_id).await?;
        match provider.update_record(record_id, &request).await {
            Ok(record) => Ok(record),
            Err(e) => Err(self.ctx.handle_provider_error(account_id, e).await),
        }
    }

    /// Deletes one DNS record.
    pub async fn delete_record(
        &self,
        account_id: &str,
        record_id: &str,
        domain_id: &str,
    ) -> CoreResult<()> {
        let provider = self.ctx.get_provider(account_id).await?;
        match provider.delete_record(record_id, domain_id).await {
            Ok(()) => Ok(()),
            Err(e) => Err(self.ctx.handle_provider_error(account_id, e).await),
        }
    }

    /// Deletes DNS records in batch.
    ///
    /// Deletions run concurrently with a bounded parallelism of `5`.
    pub async fn batch_delete_records(
        &self,
        account_id: &str,
        request: BatchDeleteRequest,
    ) -> CoreResult<BatchDeleteResult> {
        let provider = self.ctx.get_provider(account_id).await?;

        let domain_id = request.domain_id;
        let results: Vec<Result<String, (String, ProviderError)>> =
            stream::iter(request.record_ids)
                .map(|record_id| {
                    let provider = provider.clone();
                    let domain_id = domain_id.clone();
                    async move {
                        match provider.delete_record(&record_id, &domain_id).await {
                            Ok(()) => Ok(record_id),
                            Err(e) => Err((record_id, e)),
                        }
                    }
                })
                .buffer_unordered(5)
                .collect()
                .await;

        let mut success_count = 0;
        let mut failures = Vec::new();
        let mut account_marked_invalid = false;

        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err((record_id, e)) => {
                    if !account_marked_invalid && let ProviderError::InvalidCredentials { .. } = &e
                    {
                        self.ctx
                            .mark_account_invalid(account_id, "Credentials have expired")
                            .await;
                        account_marked_invalid = true;
                    }
                    failures.push(BatchDeleteFailure {
                        record_id,
                        reason: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchDeleteResult {
            success_count,
            failed_count: failures.len(),
            failures,
        })
    }
}
