//! `SeaORM` migrations for `SqliteStore`.

pub use sea_orm_migration::prelude::*;

mod m20250215_000001_create_initial_tables;

/// Migration entrypoint used by `sea_orm_migration::MigratorTrait`.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250215_000001_create_initial_tables::Migration)]
    }
}
