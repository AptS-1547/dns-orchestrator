//! `SeaORM` entity for the `domain_metadata` table.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "domain_metadata")]
/// Database row model for per-domain UI metadata.
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub account_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub domain_id: String,
    pub is_favorite: i32,
    pub tags: String,
    pub color: String,
    pub note: Option<String>,
    pub favorited_at: Option<String>,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
