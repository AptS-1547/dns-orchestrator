//! 创建 accounts 表
//!
//! 存储 DNS 服务商账户信息，凭证使用 AES-GCM 加密存储

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Account {
    Table,
    Id,
    Name,
    ProviderType,
    EncryptedCredentials,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(string(Account::Id).primary_key())
                    .col(string(Account::Name).not_null())
                    .col(string(Account::ProviderType).not_null())
                    .col(text(Account::EncryptedCredentials).not_null())
                    .col(timestamp(Account::CreatedAt).not_null())
                    .col(timestamp(Account::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await
    }
}
