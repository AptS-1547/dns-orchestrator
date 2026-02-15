//! Initial schema for `SqliteStore`.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
/// Create `accounts`, `credentials`, and `domain_metadata` tables.
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // accounts table
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Account::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Account::Name).string().not_null())
                    .col(ColumnDef::new(Account::Provider).string().not_null())
                    .col(ColumnDef::new(Account::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Account::UpdatedAt).string().not_null())
                    .col(ColumnDef::new(Account::Status).string().null())
                    .col(ColumnDef::new(Account::Error).string().null())
                    .to_owned(),
            )
            .await?;

        // credentials table
        manager
            .create_table(
                Table::create()
                    .table(Credential::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Credential::AccountId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Credential::Salt).string().not_null())
                    .col(ColumnDef::new(Credential::Nonce).string().not_null())
                    .col(ColumnDef::new(Credential::Ciphertext).string().not_null())
                    .to_owned(),
            )
            .await?;

        // domain_metadata table
        manager
            .create_table(
                Table::create()
                    .table(DomainMetadata::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DomainMetadata::AccountId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DomainMetadata::DomainId).string().not_null())
                    .col(
                        ColumnDef::new(DomainMetadata::IsFavorite)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DomainMetadata::Tags)
                            .string()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(DomainMetadata::Color)
                            .string()
                            .not_null()
                            .default("none"),
                    )
                    .col(ColumnDef::new(DomainMetadata::Note).string().null())
                    .col(ColumnDef::new(DomainMetadata::FavoritedAt).string().null())
                    .col(
                        ColumnDef::new(DomainMetadata::UpdatedAt)
                            .string()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(DomainMetadata::AccountId)
                            .col(DomainMetadata::DomainId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DomainMetadata::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Credential::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Account {
    #[sea_orm(iden = "accounts")]
    Table,
    Id,
    Name,
    Provider,
    CreatedAt,
    UpdatedAt,
    Status,
    Error,
}

#[derive(DeriveIden)]
enum Credential {
    #[sea_orm(iden = "credentials")]
    Table,
    AccountId,
    Salt,
    Nonce,
    Ciphertext,
}

#[derive(DeriveIden)]
enum DomainMetadata {
    #[sea_orm(iden = "domain_metadata")]
    Table,
    AccountId,
    DomainId,
    IsFavorite,
    Tags,
    Color,
    Note,
    FavoritedAt,
    UpdatedAt,
}
