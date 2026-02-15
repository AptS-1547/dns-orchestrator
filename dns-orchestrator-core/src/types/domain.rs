//! Domain-related types.

use serde::{Deserialize, Serialize};

use dns_orchestrator_provider::{DomainStatus, ProviderDomain, ProviderType};

use super::domain_metadata::DomainMetadata;

/// Application-layer domain model (extends provider domain with `account_id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDomain {
    /// Domain ID.
    pub id: String,
    /// Domain name.
    pub name: String,
    /// Account ID.
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// DNS provider type.
    pub provider: ProviderType,
    /// Domain status.
    pub status: DomainStatus,
    /// Number of DNS records.
    #[serde(rename = "recordCount", skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u32>,
    /// User-defined metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DomainMetadata>,
}

impl AppDomain {
    /// Converts a provider domain into an application domain.
    #[must_use]
    pub fn from_provider(provider_domain: ProviderDomain, account_id: String) -> Self {
        Self {
            id: provider_domain.id,
            name: provider_domain.name,
            account_id,
            provider: provider_domain.provider,
            status: provider_domain.status,
            record_count: provider_domain.record_count,
            metadata: None,
        }
    }
}
