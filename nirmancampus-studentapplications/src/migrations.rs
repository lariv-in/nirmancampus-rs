use sea_orm_migration::prelude::*;

use super::NirmancampusStudentApplicationsTag;

mod m20260831_000001_create_studentapplications;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260831_000001_create_studentapplications::Migration)]
    }
}

lariv_rs::define_register_migrations! {
    plugin: NirmancampusStudentApplicationsTag;
    migrator: Migrator;
}
