//! Built-in storage adapters for non-Tauri frontends.
//!
//! Choose an adapter based on runtime and deployment requirements:
//! - `KeyringCredentialStore` (`keyring-store` feature): credentials in system keychain.
//! - `SqliteStore` (`sqlite-store` feature): unified local `SQLite` persistence.

#[cfg(feature = "keyring-store")]
mod keyring_credential_store;

#[cfg(feature = "keyring-store")]
/// System keychain-backed credential store (macOS/Windows/Linux).
pub use keyring_credential_store::KeyringCredentialStore;

#[cfg(feature = "sqlite-store")]
mod sqlite;

#[cfg(feature = "sqlite-store")]
/// SQLite-backed unified store implementing all core storage traits.
pub use sqlite::SqliteStore;
