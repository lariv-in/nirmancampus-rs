use sea_orm_migration::prelude::*;

use super::NirmancampusWebsiteTag;

mod m20260831_000001_create_website;
mod m20260831_000002_create_tblfee;
mod m20260902_000003_drop_tblfee;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260831_000001_create_website::Migration),
            Box::new(m20260831_000002_create_tblfee::Migration),
            Box::new(m20260902_000003_drop_tblfee::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: NirmancampusWebsiteTag;
    migrator: Migrator;
}
