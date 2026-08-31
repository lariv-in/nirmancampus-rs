use sea_orm_migration::prelude::*;

use super::NirmancampusSessionsTag;

mod m20260831_000001_create_sessions;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260831_000001_create_sessions::Migration)]
    }
}

lariv_rs::define_register_migrations! {
    plugin: NirmancampusSessionsTag;
    migrator: Migrator;
}
