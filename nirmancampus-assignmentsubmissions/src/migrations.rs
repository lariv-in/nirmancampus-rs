use sea_orm_migration::prelude::*;

use super::NirmancampusAssignmentSubmissionsTag;

mod m20260831_000001_create_assignmentsubmissions;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20260831_000001_create_assignmentsubmissions::Migration,
        )]
    }
}

lariv_rs::define_register_migrations! {
    plugin: NirmancampusAssignmentSubmissionsTag;
    migrator: Migrator;
}
