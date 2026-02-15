//! SQLite-based unified store using `SeaORM`.
//!
//! A single `SqliteStore` implements `AccountRepository`, `CredentialStore`,
//! and `DomainMetadataRepository`, backed by a local `SQLite` database.
//! Credentials are encrypted with AES-256-GCM using a password provided at construction.

mod account_repo;
mod credential_store;
mod domain_metadata_repo;
pub(crate) mod entity;
mod migration;

use std::path::Path;

use dns_orchestrator_core::error::{CoreError, CoreResult};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use migration::Migrator;

/// SQLite-based unified store for MCP/TUI/CLI frontends.
///
/// Implements all three storage traits (`AccountRepository`, `CredentialStore`,
/// `DomainMetadataRepository`) against a single `SQLite` database file.
///
/// Credentials are encrypted at rest using AES-256-GCM with PBKDF2 key derivation.
///
/// If `encryption_password` is `None`, `CredentialStore` methods will return an error.
/// This allows using `SqliteStore` for only `AccountRepository` + `DomainMetadataRepository`.
pub struct SqliteStore {
    /// Shared `SeaORM` database connection.
    pub(crate) db: DatabaseConnection,
    /// Optional password used by `CredentialStore` encryption/decryption.
    pub(crate) encryption_password: Option<String>,
}

impl SqliteStore {
    /// Create a new `SQLite` store.
    ///
    /// - `db_path`: Path to the `SQLite` database file (created if not exists).
    /// - `encryption_password`: Password for encrypting/decrypting credentials.
    ///   Pass `None` if credential storage is not needed.
    ///
    /// # Errors
    /// Returns `CoreError::StorageError` if directory creation, database
    /// connection, or schema migration fails.
    pub async fn new(db_path: &Path, encryption_password: Option<String>) -> CoreResult<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::StorageError(format!("Failed to create directory: {e}")))?;
        }

        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
        let mut opts = ConnectOptions::new(db_url);
        opts.sqlx_logging(false);

        let db = Database::connect(opts)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to connect to SQLite: {e}")))?;

        let store = Self {
            db,
            encryption_password,
        };

        // Ensure schema is up to date before the store is used.
        Migrator::up(&store.db, None)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to run migrations: {e}")))?;

        Ok(store)
    }
}
