//! Import/export related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use dns_orchestrator_provider::ProviderType;

/// Export payload for one account (including credentials).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedAccount {
    /// Account ID
    pub id: String,
    /// Account name
    pub name: String,
    /// DNS provider type
    pub provider: ProviderType,
    /// Created timestamp.
    #[serde(with = "crate::utils::datetime")]
    pub created_at: DateTime<Utc>,
    /// Updated timestamp.
    #[serde(with = "crate::utils::datetime")]
    pub updated_at: DateTime<Utc>,
    /// Credential key-value data.
    pub credentials: HashMap<String, String>,
}

/// Export file header (plain-text section).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFileHeader {
    /// File format version.
    pub version: u32,
    /// Whether payload data is encrypted.
    pub encrypted: bool,
    /// Base64 salt used for encryption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    /// Base64 nonce/IV used for encryption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    /// Export timestamp.
    pub exported_at: String,
    /// Application version.
    pub app_version: String,
}

/// Complete export file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFile {
    /// File header.
    pub header: ExportFileHeader,
    /// Account data (Base64 ciphertext when encrypted, JSON array when plaintext).
    pub data: serde_json::Value,
}

/// Request payload for exporting accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsRequest {
    /// Account IDs to export.
    pub account_ids: Vec<String>,
    /// Whether to encrypt the exported data.
    pub encrypt: bool,
    /// Encryption password (required when `encrypt = true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Response payload for account export.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsResponse {
    /// Exported JSON content.
    pub content: String,
    /// Suggested filename.
    pub suggested_filename: String,
}

/// Request payload for importing accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAccountsRequest {
    /// Import file content.
    pub content: String,
    /// Decryption password (when file is encrypted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Import preview (for UI display before actual import).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    /// Whether the file is encrypted.
    pub encrypted: bool,
    /// Number of accounts in the file.
    pub account_count: usize,
    /// Account previews (available only when plaintext is available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<ImportPreviewAccount>>,
}

/// Account info used in import preview (without sensitive credentials).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewAccount {
    /// Account name.
    pub name: String,
    /// DNS provider type.
    pub provider: ProviderType,
    /// Whether this name conflicts with an existing account.
    pub has_conflict: bool,
}

/// Result payload for account import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    /// Number of accounts imported successfully.
    pub success_count: usize,
    /// Failed accounts and reasons.
    pub failures: Vec<ImportFailure>,
}

/// One failed import item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFailure {
    /// Account name.
    pub name: String,
    /// Failure reason.
    pub reason: String,
}
