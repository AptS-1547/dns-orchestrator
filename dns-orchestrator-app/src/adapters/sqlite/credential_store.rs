//! `CredentialStore` implementation for `SqliteStore`.
//!
//! Credentials are encrypted with AES-256-GCM before storage.
//! Uses `dns_orchestrator_core::crypto::{encrypt, decrypt}`.

use async_trait::async_trait;
use sea_orm::{ActiveValue::Set, EntityTrait, ModelTrait, TransactionTrait};
use std::collections::HashMap;

use dns_orchestrator_core::crypto;
use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::{CredentialStore, CredentialsMap};
use dns_orchestrator_core::types::ProviderCredentials;

use super::SqliteStore;
use super::entity::credential;

impl SqliteStore {
    /// Return the configured encryption password or an explicit credential error.
    fn get_encryption_password(&self) -> CoreResult<&str> {
        self.encryption_password.as_deref().ok_or_else(|| {
            CoreError::CredentialError("Encryption password not configured for SqliteStore".into())
        })
    }

    /// Serialize and encrypt provider credentials for database storage.
    fn encrypt_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> CoreResult<(String, String, String)> {
        let password = self.get_encryption_password()?;
        let json = serde_json::to_string(credentials)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        crypto::encrypt(json.as_bytes(), password)
    }

    /// Decrypt and deserialize provider credentials from a database row.
    fn decrypt_credentials(&self, model: &credential::Model) -> CoreResult<ProviderCredentials> {
        let password = self.get_encryption_password()?;
        let plaintext = crypto::decrypt(&model.ciphertext, password, &model.salt, &model.nonce)?;
        let json = String::from_utf8(plaintext)
            .map_err(|e| CoreError::SerializationError(format!("Invalid UTF-8: {e}")))?;
        serde_json::from_str(&json)
            .map_err(|e| CoreError::SerializationError(format!("Invalid credentials JSON: {e}")))
    }
}

#[async_trait]
impl CredentialStore for SqliteStore {
    async fn load_all(&self) -> CoreResult<CredentialsMap> {
        let rows = credential::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query credentials: {e}")))?;

        let mut map = HashMap::new();
        for row in &rows {
            match self.decrypt_credentials(row) {
                Ok(creds) => {
                    map.insert(row.account_id.clone(), creds);
                }
                Err(e) => {
                    log::warn!(
                        "Failed to decrypt credentials for account {}: {e}",
                        row.account_id
                    );
                }
            }
        }

        Ok(map)
    }

    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to begin transaction: {e}")))?;

        credential::Entity::delete_many()
            .exec(&txn)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to clear credentials: {e}")))?;

        for (account_id, creds) in credentials {
            let (salt, nonce, ciphertext) = self.encrypt_credentials(creds)?;
            let active_model = credential::ActiveModel {
                account_id: Set(account_id.clone()),
                salt: Set(salt),
                nonce: Set(nonce),
                ciphertext: Set(ciphertext),
            };

            credential::Entity::insert(active_model)
                .exec(&txn)
                .await
                .map_err(|e| CoreError::StorageError(format!("Failed to save credential: {e}")))?;
        }

        txn.commit()
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to commit transaction: {e}")))?;

        log::info!("Saved {} credentials to SQLite", credentials.len());
        Ok(())
    }

    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        let row = credential::Entity::find_by_id(account_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query credential: {e}")))?;

        match row {
            Some(r) => Ok(Some(self.decrypt_credentials(&r)?)),
            None => Ok(None),
        }
    }

    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()> {
        let (salt, nonce, ciphertext) = self.encrypt_credentials(credentials)?;

        let active_model = credential::ActiveModel {
            account_id: Set(account_id.to_string()),
            salt: Set(salt),
            nonce: Set(nonce),
            ciphertext: Set(ciphertext),
        };

        credential::Entity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(credential::Column::AccountId)
                    .update_columns([
                        credential::Column::Salt,
                        credential::Column::Nonce,
                        credential::Column::Ciphertext,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to save credential: {e}")))?;

        log::info!("Credentials saved for account: {account_id}");
        Ok(())
    }

    async fn remove(&self, account_id: &str) -> CoreResult<()> {
        let model = credential::Entity::find_by_id(account_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query credential: {e}")))?;

        if let Some(m) = model {
            m.delete(&self.db).await.map_err(|e| {
                CoreError::StorageError(format!("Failed to delete credential: {e}"))
            })?;
        }

        log::info!("Credentials deleted for account: {account_id}");
        Ok(())
    }

    async fn load_raw_json(&self) -> CoreResult<String> {
        let all = self.load_all().await?;
        serde_json::to_string(&all).map_err(|e| CoreError::SerializationError(e.to_string()))
    }

    async fn save_raw_json(&self, json: &str) -> CoreResult<()> {
        let credentials: CredentialsMap =
            serde_json::from_str(json).map_err(|e| CoreError::SerializationError(e.to_string()))?;
        self.save_all(&credentials).await
    }
}
