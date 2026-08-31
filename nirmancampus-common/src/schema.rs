//! PostgreSQL schema helpers matching Go goose migrations.

use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

pub fn is_postgres(manager: &SchemaManager<'_>) -> bool {
    manager.get_connection().get_database_backend() == DbBackend::Postgres
}

pub fn filesystem_nodes_fk<T: Iden + 'static, C: Iden + 'static>(
    name: &str,
    from_table: T,
    from_col: C,
) -> ForeignKeyCreateStatement {
    ForeignKey::create()
        .name(name)
        .from(from_table, from_col)
        .to(FilesystemNodes::Table, FilesystemNodes::Id)
        .on_delete(ForeignKeyAction::SetNull)
        .to_owned()
}

#[derive(Iden)]
pub enum FilesystemNodes {
    Table,
    Id,
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
}
