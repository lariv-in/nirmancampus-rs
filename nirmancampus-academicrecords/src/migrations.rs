use sea_orm_migration::prelude::*;

use super::NirmancampusAcademicRecordsTag;

mod m20260831_000001_create_academicrecords;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260831_000001_create_academicrecords::Migration)]
    }
}

lariv_rs::define_register_migrations! {
    plugin: NirmancampusAcademicRecordsTag;
    migrator: Migrator;
}
