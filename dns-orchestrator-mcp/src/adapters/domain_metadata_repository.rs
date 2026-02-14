//! `NoOp` domain metadata repository
//!
//! A no-op implementation that returns empty/default values.

use async_trait::async_trait;
use dns_orchestrator_core::error::CoreResult;
use dns_orchestrator_core::traits::DomainMetadataRepository;
use dns_orchestrator_core::types::{
    DomainMetadata, DomainMetadataKey, DomainMetadataUpdate,
};
use std::collections::HashMap;

/// `NoOp` implementation of `DomainMetadataRepository`.
///
/// This implementation returns empty/default values for all operations,
/// effectively disabling domain metadata functionality for the MCP server.
/// This is intentional as metadata management is not needed for AI agent use cases.
pub struct NoOpDomainMetadataRepository;

impl NoOpDomainMetadataRepository {
    /// Create a new `NoOp` repository instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NoOpDomainMetadataRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainMetadataRepository for NoOpDomainMetadataRepository {
    async fn find_by_key(&self, _key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>> {
        Ok(None)
    }

    async fn find_by_keys(
        &self,
        _keys: &[DomainMetadataKey],
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>> {
        Ok(HashMap::new())
    }

    async fn save(
        &self,
        _key: &DomainMetadataKey,
        _metadata: &DomainMetadata,
    ) -> CoreResult<()> {
        Ok(())
    }

    async fn batch_save(
        &self,
        _entries: &[(DomainMetadataKey, DomainMetadata)],
    ) -> CoreResult<()> {
        Ok(())
    }

    async fn update(
        &self,
        _key: &DomainMetadataKey,
        _update: &DomainMetadataUpdate,
    ) -> CoreResult<()> {
        Ok(())
    }

    async fn delete(&self, _key: &DomainMetadataKey) -> CoreResult<()> {
        Ok(())
    }

    async fn delete_by_account(&self, _account_id: &str) -> CoreResult<()> {
        Ok(())
    }

    async fn find_favorites_by_account(
        &self,
        _account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>> {
        Ok(Vec::new())
    }

    async fn find_by_tag(&self, _tag: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        Ok(Vec::new())
    }

    async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        Ok(Vec::new())
    }
}
