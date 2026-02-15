//! `AccountRepository` implementation for `SqliteStore`.

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};

use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::AccountRepository;
use dns_orchestrator_core::types::{Account, AccountStatus};

use super::SqliteStore;
use super::entity::account;

impl account::Model {
    /// Convert a `SeaORM` row model into a domain `Account`.
    ///
    /// String-backed fields are parsed into strongly typed values.
    fn into_account(self) -> CoreResult<Account> {
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|e| CoreError::SerializationError(format!("Invalid created_at: {e}")))?
            .with_timezone(&chrono::Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|e| CoreError::SerializationError(format!("Invalid updated_at: {e}")))?
            .with_timezone(&chrono::Utc);
        let provider = serde_json::from_value(serde_json::Value::String(self.provider))
            .map_err(|e| CoreError::SerializationError(format!("Invalid provider: {e}")))?;
        let status: Option<AccountStatus> = self
            .status
            .map(|s| serde_json::from_value(serde_json::Value::String(s)))
            .transpose()
            .map_err(|e| CoreError::SerializationError(format!("Invalid status: {e}")))?;

        Ok(Account {
            id: self.id,
            name: self.name,
            provider,
            created_at,
            updated_at,
            status,
            error: self.error,
        })
    }
}

/// Convert a domain `Account` into a `SeaORM` active model for upsert.
fn account_to_active_model(account: &Account) -> CoreResult<account::ActiveModel> {
    let provider_str = serde_json::to_value(&account.provider)
        .map_err(|e| CoreError::SerializationError(e.to_string()))?
        .as_str()
        .map(String::from)
        .ok_or_else(|| {
            CoreError::SerializationError("Provider did not serialize to a string".into())
        })?;

    let status_str = account
        .status
        .as_ref()
        .map(|s| {
            serde_json::to_value(s)
                .map_err(|e| CoreError::SerializationError(e.to_string()))
                .and_then(|v| {
                    v.as_str().map(String::from).ok_or_else(|| {
                        CoreError::SerializationError("Status did not serialize to a string".into())
                    })
                })
        })
        .transpose()?;

    Ok(account::ActiveModel {
        id: Set(account.id.clone()),
        name: Set(account.name.clone()),
        provider: Set(provider_str),
        created_at: Set(account.created_at.to_rfc3339()),
        updated_at: Set(account.updated_at.to_rfc3339()),
        status: Set(status_str),
        error: Set(account.error.clone()),
    })
}

#[async_trait]
impl AccountRepository for SqliteStore {
    async fn find_all(&self) -> CoreResult<Vec<Account>> {
        let rows = account::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query accounts: {e}")))?;

        rows.into_iter().map(account::Model::into_account).collect()
    }

    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>> {
        let row = account::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query account: {e}")))?;

        row.map(account::Model::into_account).transpose()
    }

    async fn save(&self, account: &Account) -> CoreResult<()> {
        let active_model = account_to_active_model(account)?;

        account::Entity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(account::Column::Id)
                    .update_columns([
                        account::Column::Name,
                        account::Column::Provider,
                        account::Column::UpdatedAt,
                        account::Column::Status,
                        account::Column::Error,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to save account: {e}")))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        let res = account::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to delete account: {e}")))?;

        if res.rows_affected == 0 {
            Err(CoreError::AccountNotFound(id.to_string()))
        } else {
            Ok(())
        }
    }

    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()> {
        for account in accounts {
            self.save(account).await?;
        }
        Ok(())
    }

    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        let status_str = serde_json::to_value(&status)
            .ok()
            .and_then(|v| v.as_str().map(String::from));

        let model = account::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query account: {e}")))?;

        match model {
            Some(_) => {
                let active = account::ActiveModel {
                    id: Set(id.to_string()),
                    status: Set(status_str),
                    error: Set(error),
                    updated_at: Set(chrono::Utc::now().to_rfc3339()),
                    ..Default::default()
                };
                active.update(&self.db).await.map_err(|e| {
                    CoreError::StorageError(format!("Failed to update status: {e}"))
                })?;
                Ok(())
            }
            None => Err(CoreError::AccountNotFound(id.to_string())),
        }
    }
}
