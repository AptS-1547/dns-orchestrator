//! Provider metadata service
//!
//! Exposes static DNS provider metadata (stateless).

use dns_orchestrator_provider::get_all_provider_metadata;

use crate::types::ProviderMetadata;

/// Stateless provider metadata service.
pub struct ProviderMetadataService;

impl ProviderMetadataService {
    /// Creates a provider metadata service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Returns all supported provider metadata.
    pub fn list_providers(&self) -> Vec<ProviderMetadata> {
        get_all_provider_metadata()
    }
}

impl Default for ProviderMetadataService {
    fn default() -> Self {
        Self::new()
    }
}
