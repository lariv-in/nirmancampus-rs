use sea_orm_migration::prelude::*;

use super::NirmancampusStudentFeesTag;

mod m20260902_000001_create_student_fees_preferences;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20260902_000001_create_student_fees_preferences::Migration,
        )]
    }
}

lariv_rs::define_register_migrations! {
    plugin: NirmancampusStudentFeesTag;
    migrator: Migrator;
}
