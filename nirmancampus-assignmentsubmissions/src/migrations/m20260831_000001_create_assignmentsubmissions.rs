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
                    .table(AssignmentSubmissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AssignmentSubmissions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::CreatedAt).timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::UpdatedAt).timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::DeletedAt).timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::AssignmentTitle)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::MaxMarks)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::SubmissionStatus)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::Marks)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::CourseId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissions::AcademicRecordId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_assignment_submissions_course")
                            .from(
                                AssignmentSubmissions::Table,
                                AssignmentSubmissions::CourseId,
                            )
                            .to(Courses::Table, Courses::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_assignment_submissions_academic_record")
                            .from(
                                AssignmentSubmissions::Table,
                                AssignmentSubmissions::AcademicRecordId,
                            )
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
                    .name("idx_assignment_submissions_deleted_at")
                    .table(AssignmentSubmissions::Table)
                    .col(AssignmentSubmissions::DeletedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_assignment_submissions_assignment_title")
                    .table(AssignmentSubmissions::Table)
                    .col(AssignmentSubmissions::AssignmentTitle)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_assignment_submissions_submission_status")
                    .table(AssignmentSubmissions::Table)
                    .col(AssignmentSubmissions::SubmissionStatus)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_assignment_submissions_course_id")
                    .table(AssignmentSubmissions::Table)
                    .col(AssignmentSubmissions::CourseId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_assignment_submissions_academic_record_id")
                    .table(AssignmentSubmissions::Table)
                    .col(AssignmentSubmissions::AcademicRecordId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AssignmentSubmissionAssets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AssignmentSubmissionAssets::AssignmentSubmissionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AssignmentSubmissionAssets::VNodeId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(AssignmentSubmissionAssets::AssignmentSubmissionId)
                            .col(AssignmentSubmissionAssets::VNodeId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_assignment_submission_assets_submission")
                            .from(
                                AssignmentSubmissionAssets::Table,
                                AssignmentSubmissionAssets::AssignmentSubmissionId,
                            )
                            .to(AssignmentSubmissions::Table, AssignmentSubmissions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_assignment_submission_assets_vnode")
                            .from(
                                AssignmentSubmissionAssets::Table,
                                AssignmentSubmissionAssets::VNodeId,
                            )
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
            .drop_table(
                Table::drop()
                    .table(AssignmentSubmissionAssets::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(AssignmentSubmissions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AssignmentSubmissions {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    AssignmentTitle,
    MaxMarks,
    SubmissionStatus,
    Marks,
    CourseId,
    AcademicRecordId,
}

#[derive(Iden)]
enum AssignmentSubmissionAssets {
    Table,
    AssignmentSubmissionId,
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
