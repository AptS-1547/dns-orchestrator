//! Account-related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use dns_orchestrator_provider::{ProviderCredentials, ProviderType};

/// Account status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    /// Account is active.
    Active,
    /// Account is in error state (for example invalid credentials).
    Error,
}

/// Account metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account ID (UUID)
    pub id: String,
    /// Account name
    pub name: String,
    /// DNS provider type
    pub provider: ProviderType,
    /// Created timestamp.
    #[serde(rename = "createdAt")]
    #[serde(with = "crate::utils::datetime")]
    pub created_at: DateTime<Utc>,
    /// Updated timestamp.
    #[serde(rename = "updatedAt")]
    #[serde(with = "crate::utils::datetime")]
    pub updated_at: DateTime<Utc>,
    /// Account status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AccountStatus>,
    /// Error message (when the status is Error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request payload for creating an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    /// Account name
    pub name: String,
    /// DNS provider type
    pub provider: ProviderType,
    /// Structured credentials.
    pub credentials: ProviderCredentials,
}

/// Request payload for updating an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    /// Account ID
    pub id: String,
    /// New account name (optional)
    pub name: Option<String>,
    /// New credentials (optional, will overwrite the original credentials when provided)
    pub credentials: Option<ProviderCredentials>,
}
