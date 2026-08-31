use sea_orm_migration::prelude::*;

use nirmancampus_common::schema::FilesystemNodes;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExamRegistrations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExamRegistrations::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ExamRegistrations::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ExamRegistrations::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ExamRegistrations::DeletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(ExamRegistrations::ExamTitle)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExamRegistrations::MaxMarks)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExamRegistrations::RegistrationStatus)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExamRegistrations::Marks)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExamRegistrations::Fee)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ExamRegistrations::CourseId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExamRegistrations::AcademicRecordId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_exam_registrations_course")
                            .from(ExamRegistrations::Table, ExamRegistrations::CourseId)
                            .to(Courses::Table, Courses::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_exam_registrations_academic_record")
                            .from(ExamRegistrations::Table, ExamRegistrations::AcademicRecordId)
                            .to(AcademicRecords::Table, AcademicRecords::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_exam_registrations_deleted_at")
                    .table(ExamRegistrations::Table)
                    .col(ExamRegistrations::DeletedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_exam_registrations_exam_title")
                    .table(ExamRegistrations::Table)
                    .col(ExamRegistrations::ExamTitle)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_exam_registrations_registration_status")
                    .table(ExamRegistrations::Table)
                    .col(ExamRegistrations::RegistrationStatus)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_exam_registrations_course_id")
                    .table(ExamRegistrations::Table)
                    .col(ExamRegistrations::CourseId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_exam_registrations_academic_record_id")
                    .table(ExamRegistrations::Table)
                    .col(ExamRegistrations::AcademicRecordId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ExamRegistrationAssets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExamRegistrationAssets::ExamRegistrationId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExamRegistrationAssets::VNodeId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ExamRegistrationAssets::ExamRegistrationId)
                            .col(ExamRegistrationAssets::VNodeId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_exam_registration_assets_exam")
                            .from(
                                ExamRegistrationAssets::Table,
                                ExamRegistrationAssets::ExamRegistrationId,
                            )
                            .to(ExamRegistrations::Table, ExamRegistrations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_exam_registration_assets_vnode")
                            .from(ExamRegistrationAssets::Table, ExamRegistrationAssets::VNodeId)
                            .to(FilesystemNodes::Table, FilesystemNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExamRegistrationAssets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ExamRegistrations::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ExamRegistrations {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    ExamTitle,
    MaxMarks,
    RegistrationStatus,
    Marks,
    Fee,
    CourseId,
    AcademicRecordId,
}

#[derive(Iden)]
enum ExamRegistrationAssets {
    Table,
    ExamRegistrationId,
    VNodeId,
}

#[derive(Iden)]
enum Courses {
    Table,
    Id,
}

#[derive(Iden)]
enum AcademicRecords {
    Table,
    Id,
}
