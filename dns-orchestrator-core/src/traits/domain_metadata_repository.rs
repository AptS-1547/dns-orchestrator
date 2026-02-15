//! Domain metadata persistence abstraction.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::CoreResult;
use crate::types::{DomainMetadata, DomainMetadataKey, DomainMetadataUpdate};

/// Repository for domain metadata.
///
/// Platform implementations:
/// - Tauri: `TauriDomainMetadataRepository` (tauri-plugin-store)
/// - Actix-Web: `DatabaseDomainMetadataRepository` (`SeaORM`)
#[async_trait]
pub trait DomainMetadataRepository: Send + Sync {
    /// Returns metadata for one domain.
    ///
    /// # Returns
    /// * `Some(metadata)` - Metadata exists.
    /// * `None` - Metadata does not exist (callers may use defaults).
    async fn find_by_key(&self, key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>>;

    /// Returns metadata for multiple domains in one call.
    ///
    /// # Arguments
    /// * `keys` - Domain metadata keys.
    ///
    /// # Returns
    /// * A key-value map containing only existing entries.
    async fn find_by_keys(
        &self,
        keys: &[DomainMetadataKey],
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>>;

    /// Saves or updates metadata.
    ///
    /// # Arguments
    /// * `key` - Domain metadata key.
    /// * `metadata` - Metadata payload.
    ///
    /// # Note
    /// If `metadata.is_empty()` is `true`, implementations should delete the stored entry.
    async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata) -> CoreResult<()>;

    /// Saves metadata for multiple domains in batch.
    ///
    /// # Arguments
    /// * `entries` - List of `(key, metadata)` pairs.
    ///
    /// # Note
    /// - Implementations should optimize this path to avoid repeated writes.
    /// - If `metadata.is_empty()` is `true`, the corresponding entry should be deleted.
    async fn batch_save(&self, entries: &[(DomainMetadataKey, DomainMetadata)]) -> CoreResult<()>;

    /// Applies a partial metadata update.
    async fn update(
        &self,
        key: &DomainMetadataKey,
        update: &DomainMetadataUpdate,
    ) -> CoreResult<()>;

    /// Deletes metadata for one domain.
    async fn delete(&self, key: &DomainMetadataKey) -> CoreResult<()>;

    /// Deletes all metadata under one account.
    async fn delete_by_account(&self, account_id: &str) -> CoreResult<()>;

    /// Returns all favorited domain keys under one account.
    async fn find_favorites_by_account(
        &self,
        account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>>;

    /// Finds domains by tag.
    async fn find_by_tag(&self, tag: &str) -> CoreResult<Vec<DomainMetadataKey>>;

    /// Returns all tags currently in use (deduplicated and sorted).
    async fn list_all_tags(&self) -> CoreResult<Vec<String>>;
}
