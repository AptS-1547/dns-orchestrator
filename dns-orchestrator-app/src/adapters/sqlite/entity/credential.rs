//! `SeaORM` entity for the `credentials` table.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "credentials")]
/// Database row model for encrypted provider credentials.
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub account_id: String,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
