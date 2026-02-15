//! Unified core error definitions.

use serde::Serialize;
use thiserror::Error;

// Re-export provider library error types.
pub use dns_orchestrator_provider::{CredentialValidationError, ProviderError};

/// Error type for the core layer.
#[derive(Error, Debug, Serialize)]
#[serde(tag = "code", content = "details")]
pub enum CoreError {
    /// Provider was not found.
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    /// Account was not found.
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// Domain was not found.
    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    /// DNS record was not found.
    #[error("Record not found: {0}")]
    RecordNotFound(String),

    /// Credential storage error.
    #[error("Credential error: {0}")]
    CredentialError(String),

    /// Credential validation error (structured, field-level details supported).
    #[error("{0}")]
    CredentialValidation(CredentialValidationError),

    /// Provider API error.
    #[error("API error: {provider} - {message}")]
    ApiError { provider: String, message: String },

    /// Invalid credentials.
    #[error("Invalid credentials for: {0}")]
    InvalidCredentials(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Import/export error.
    #[error("Import/Export error: {0}")]
    ImportExportError(String),

    /// No account selected (during export).
    #[error("No accounts selected")]
    NoAccountsSelected,

    /// Unsupported file version (during import).
    #[error("Unsupported file version")]
    UnsupportedFileVersion,

    /// Storage layer error.
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Network error.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Data format migration required (v1.7.0 credential format upgrade).
    #[error("Credential data migration required")]
    MigrationRequired,

    /// Migration failed.
    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    /// Provider library error.
    #[error("{0}")]
    Provider(#[from] ProviderError),
}

impl CoreError {
    /// Returns whether this error is expected (user input, missing resource, etc.).
    ///
    /// Use `warn` when this returns `true`, and `error` otherwise.
    /// Keep this method updated when adding new variants.
    #[must_use]
    pub fn is_expected(&self) -> bool {
        match self {
            Self::AccountNotFound(_)
            | Self::DomainNotFound(_)
            | Self::RecordNotFound(_)
            | Self::ProviderNotFound(_)
            | Self::ValidationError(_)
            | Self::NoAccountsSelected
            | Self::UnsupportedFileVersion
            | Self::CredentialValidation(_)
            | Self::InvalidCredentials(_) => true,
            Self::Provider(e) => e.is_expected(),
            _ => false,
        }
    }
}

/// `Result` alias used by the core layer.
pub type CoreResult<T> = std::result::Result<T, CoreError>;
