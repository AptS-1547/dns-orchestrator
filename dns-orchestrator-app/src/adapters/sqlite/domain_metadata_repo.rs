//! `DomainMetadataRepository` implementation for `SqliteStore`.

use async_trait::async_trait;
use std::collections::HashMap;

use sea_orm::{
    ActiveValue::Set, Condition,
    ColumnTrait, EntityTrait, FromQueryResult, ModelTrait, QueryFilter, QueryTrait,
    sea_query::{Alias, Expr, ExprTrait, Func, IntoIden, Query, TableRef},
};

use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::DomainMetadataRepository;
use dns_orchestrator_core::types::{DomainMetadata, DomainMetadataKey, DomainMetadataUpdate};

use super::SqliteStore;
use super::entity::domain_metadata;

impl domain_metadata::Model {
    /// Convert a `SeaORM` row model into `(DomainMetadataKey, DomainMetadata)`.
    fn into_key_and_metadata(self) -> CoreResult<(DomainMetadataKey, DomainMetadata)> {
        let tags: Vec<String> = serde_json::from_str(&self.tags)
            .map_err(|e| CoreError::SerializationError(format!("Invalid tags JSON: {e}")))?;

        let favorited_at = self
            .favorited_at
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .map_err(|e| {
                        CoreError::SerializationError(format!("Invalid favorited_at: {e}"))
                    })
            })
            .transpose()?;

        let updated_at = chrono::DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|e| CoreError::SerializationError(format!("Invalid updated_at: {e}")))?
            .with_timezone(&chrono::Utc);

        let key = DomainMetadataKey {
            account_id: self.account_id,
            domain_id: self.domain_id,
        };

        let metadata = DomainMetadata {
            is_favorite: self.is_favorite != 0,
            tags,
            color: self.color,
            note: self.note,
            favorited_at,
            updated_at,
        };

        Ok((key, metadata))
    }
}

/// Build `CROSS JOIN json_each(domain_metadata.tags) AS je` as a `TableRef`.
fn json_each_tags() -> TableRef {
    let func_call = Func::cust(Alias::new("json_each")).arg(Expr::col((
        domain_metadata::Entity,
        domain_metadata::Column::Tags,
    )));
    TableRef::FunctionCall(func_call, Alias::new("je").into_iden())
}

/// Convert domain metadata into a `SeaORM` active model for upsert.
fn metadata_to_active_model(
    key: &DomainMetadataKey,
    metadata: &DomainMetadata,
) -> CoreResult<domain_metadata::ActiveModel> {
    let tags_json = serde_json::to_string(&metadata.tags)
        .map_err(|e| CoreError::SerializationError(e.to_string()))?;
    let favorited_at = metadata.favorited_at.map(|dt| dt.to_rfc3339());

    Ok(domain_metadata::ActiveModel {
        account_id: Set(key.account_id.clone()),
        domain_id: Set(key.domain_id.clone()),
        is_favorite: Set(i32::from(metadata.is_favorite)),
        tags: Set(tags_json),
        color: Set(metadata.color.clone()),
        note: Set(metadata.note.clone()),
        favorited_at: Set(favorited_at),
        updated_at: Set(metadata.updated_at.to_rfc3339()),
    })
}

impl SqliteStore {
    /// Insert or update metadata for a key, or delete the row if metadata is empty.
    async fn upsert_metadata(
        &self,
        key: &DomainMetadataKey,
        metadata: &DomainMetadata,
    ) -> CoreResult<()> {
        if metadata.is_empty() {
            domain_metadata::Entity::delete_many()
                .filter(domain_metadata::Column::AccountId.eq(&key.account_id))
                .filter(domain_metadata::Column::DomainId.eq(&key.domain_id))
                .exec(&self.db)
                .await
                .map_err(|e| CoreError::StorageError(format!("Failed to delete metadata: {e}")))?;
            return Ok(());
        }

        let active_model = metadata_to_active_model(key, metadata)?;

        domain_metadata::Entity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    domain_metadata::Column::AccountId,
                    domain_metadata::Column::DomainId,
                ])
                .update_columns([
                    domain_metadata::Column::IsFavorite,
                    domain_metadata::Column::Tags,
                    domain_metadata::Column::Color,
                    domain_metadata::Column::Note,
                    domain_metadata::Column::FavoritedAt,
                    domain_metadata::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to upsert metadata: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl DomainMetadataRepository for SqliteStore {
    async fn find_by_key(&self, key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>> {
        let row =
            domain_metadata::Entity::find_by_id((key.account_id.clone(), key.domain_id.clone()))
                .one(&self.db)
                .await
                .map_err(|e| CoreError::StorageError(format!("Failed to query metadata: {e}")))?;

        match row {
            Some(r) => {
                let (_, metadata) = r.into_key_and_metadata()?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    async fn find_by_keys(
        &self,
        keys: &[DomainMetadataKey],
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut condition = Condition::any();
        for key in keys {
            condition = condition.add(
                Condition::all()
                    .add(domain_metadata::Column::AccountId.eq(&key.account_id))
                    .add(domain_metadata::Column::DomainId.eq(&key.domain_id)),
            );
        }

        let rows = domain_metadata::Entity::find()
            .filter(condition)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query metadata: {e}")))?;

        let mut result = HashMap::new();
        for row in rows {
            let (key, metadata) = row.into_key_and_metadata()?;
            result.insert(key, metadata);
        }

        Ok(result)
    }

    async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata) -> CoreResult<()> {
        self.upsert_metadata(key, metadata).await
    }

    async fn batch_save(&self, entries: &[(DomainMetadataKey, DomainMetadata)]) -> CoreResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        for (key, metadata) in entries {
            self.upsert_metadata(key, metadata).await?;
        }

        log::info!("Batch saved {} domain metadata entries", entries.len());
        Ok(())
    }

    async fn update(
        &self,
        key: &DomainMetadataKey,
        update: &DomainMetadataUpdate,
    ) -> CoreResult<()> {
        let mut metadata = self.find_by_key(key).await?.unwrap_or_default();
        update.apply_to(&mut metadata);
        self.upsert_metadata(key, &metadata).await
    }

    async fn delete(&self, key: &DomainMetadataKey) -> CoreResult<()> {
        let model =
            domain_metadata::Entity::find_by_id((key.account_id.clone(), key.domain_id.clone()))
                .one(&self.db)
                .await
                .map_err(|e| CoreError::StorageError(format!("Failed to query metadata: {e}")))?;

        if let Some(m) = model {
            m.delete(&self.db)
                .await
                .map_err(|e| CoreError::StorageError(format!("Failed to delete metadata: {e}")))?;
        }

        Ok(())
    }

    async fn delete_by_account(&self, account_id: &str) -> CoreResult<()> {
        domain_metadata::Entity::delete_many()
            .filter(domain_metadata::Column::AccountId.eq(account_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                CoreError::StorageError(format!("Failed to delete metadata by account: {e}"))
            })?;

        Ok(())
    }

    async fn find_favorites_by_account(
        &self,
        account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>> {
        let rows = domain_metadata::Entity::find()
            .filter(domain_metadata::Column::AccountId.eq(account_id))
            .filter(domain_metadata::Column::IsFavorite.eq(1))
            .all(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query favorites: {e}")))?;

        rows.into_iter()
            .map(|r| {
                let (key, _) = r.into_key_and_metadata()?;
                Ok(key)
            })
            .collect()
    }

    async fn find_by_tag(&self, tag: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        let je_value = (Alias::new("je"), Alias::new("value"));

        let mut select = domain_metadata::Entity::find();
        QueryTrait::query(&mut select)
            .cross_join(json_each_tags())
            .and_where(Expr::col(je_value).eq(tag))
            .distinct();

        let rows = select
            .all(&self.db)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to query by tag: {e}")))?;

        rows.into_iter()
            .map(|r| {
                let (key, _) = r.into_key_and_metadata()?;
                Ok(key)
            })
            .collect()
    }

    async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        #[derive(Debug, FromQueryResult)]
        struct TagRow {
            value: String,
        }

        let je_value = (Alias::new("je"), Alias::new("value"));

        let query = Query::select()
            .from(domain_metadata::Entity)
            .cross_join(json_each_tags())
            .column(je_value.clone())
            .distinct()
            .order_by(je_value, sea_orm::sea_query::Order::Asc)
            .to_owned();

        let rows = TagRow::find_by_statement(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            query.to_string(sea_orm::sea_query::SqliteQueryBuilder),
        ))
        .all(&self.db)
        .await
        .map_err(|e| CoreError::StorageError(format!("Failed to list tags: {e}")))?;

        Ok(rows.into_iter().map(|r| r.value).collect())
    }
}
