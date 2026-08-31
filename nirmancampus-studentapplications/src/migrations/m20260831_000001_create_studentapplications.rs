use sea_orm_migration::prelude::*;

use nirmancampus_common::schema::{FilesystemNodes, Users};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StudentApplications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentApplications::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(StudentApplications::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(StudentApplications::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(StudentApplications::DeletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(StudentApplications::ProgramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(StudentApplications::CreatedById).big_integer())
                    .col(
                        ColumnDef::new(StudentApplications::StudentName)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(StudentApplications::Email).text())
                    .col(ColumnDef::new(StudentApplications::Dob).date())
                    .col(ColumnDef::new(StudentApplications::MotherName).text())
                    .col(ColumnDef::new(StudentApplications::FatherName).text())
                    .col(ColumnDef::new(StudentApplications::Category).text())
                    .col(ColumnDef::new(StudentApplications::Address).text())
                    .col(ColumnDef::new(StudentApplications::Mobile).text())
                    .col(ColumnDef::new(StudentApplications::PhotoId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_applications_program")
                            .from(StudentApplications::Table, StudentApplications::ProgramId)
                            .to(Programs::Table, Programs::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_applications_created_by")
                            .from(StudentApplications::Table, StudentApplications::CreatedById)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_applications_photo")
                            .from(StudentApplications::Table, StudentApplications::PhotoId)
                            .to(FilesystemNodes::Table, FilesystemNodes::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_student_applications_deleted_at")
                    .table(StudentApplications::Table)
                    .col(StudentApplications::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_student_applications_program_id")
                    .table(StudentApplications::Table)
                    .col(StudentApplications::ProgramId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(StudentApplicationDocuments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentApplicationDocuments::StudentApplicationId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentApplicationDocuments::VNodeId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(StudentApplicationDocuments::StudentApplicationId)
                            .col(StudentApplicationDocuments::VNodeId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_application_documents_app")
                            .from(
                                StudentApplicationDocuments::Table,
                                StudentApplicationDocuments::StudentApplicationId,
                            )
                            .to(StudentApplications::Table, StudentApplications::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_application_documents_vnode")
                            .from(
                                StudentApplicationDocuments::Table,
                                StudentApplicationDocuments::VNodeId,
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
                    .table(StudentApplicationDocuments::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(StudentApplications::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum StudentApplications {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    ProgramId,
    CreatedById,
    StudentName,
    Email,
    Dob,
    MotherName,
    FatherName,
    Category,
    Address,
    Mobile,
    PhotoId,
}

#[derive(Iden)]
enum StudentApplicationDocuments {
    Table,
    StudentApplicationId,
    VNodeId,
}

#[derive(Iden)]
enum Programs {
    Table,
    Id,
}
