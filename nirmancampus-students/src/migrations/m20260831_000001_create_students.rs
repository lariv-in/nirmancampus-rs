use sea_orm::Statement;
use sea_orm_migration::prelude::*;

use nirmancampus_common::schema::{FilesystemNodes, filesystem_nodes_fk};

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn execute(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute(Statement::from_string(
            manager.get_connection().get_database_backend(),
            sql.to_string(),
        ))
        .await
        .map(|_| ())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Students::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Students::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Students::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Students::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Students::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Students::Name).string_len(255).default(""))
                    .col(ColumnDef::new(Students::Email).string_len(255).default(""))
                    .col(ColumnDef::new(Students::Phone).string_len(64).default(""))
                    .col(ColumnDef::new(Students::StudentNo).text().not_null().unique_key())
                    .col(ColumnDef::new(Students::AadharCard).string_len(32).default(""))
                    .col(ColumnDef::new(Students::AbcId).string_len(64).default(""))
                    .col(ColumnDef::new(Students::Dob).date())
                    .col(ColumnDef::new(Students::MotherName).string_len(255).default(""))
                    .col(
                        ColumnDef::new(Students::FathersName)
                            .string_len(255)
                            .default(""),
                    )
                    .col(ColumnDef::new(Students::Category).string_len(100).default(""))
                    .col(
                        ColumnDef::new(Students::Handicapped)
                            .boolean()
                            .default(false),
                    )
                    .col(ColumnDef::new(Students::Address).text())
                    .col(ColumnDef::new(Students::Remarks).text())
                    .col(ColumnDef::new(Students::PhotoId).big_integer())
                    .foreign_key(&mut filesystem_nodes_fk(
                        "fk_students_photo_id",
                        Students::Table,
                        Students::PhotoId,
                    ))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_students_deleted_at")
                    .table(Students::Table)
                    .col(Students::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_students_email")
                    .table(Students::Table)
                    .col(Students::Email)
                    .to_owned(),
            )
            .await?;

        execute(manager, "ALTER TABLE students DROP COLUMN IF EXISTS user_id").await?;

        manager
            .create_table(
                Table::create()
                    .table(StudentDocuments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentDocuments::StudentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StudentDocuments::VNodeId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(StudentDocuments::StudentId)
                            .col(StudentDocuments::VNodeId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_documents_student_id")
                            .from(StudentDocuments::Table, StudentDocuments::StudentId)
                            .to(Students::Table, Students::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_documents_v_node_id")
                            .from(StudentDocuments::Table, StudentDocuments::VNodeId)
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
            .drop_table(Table::drop().table(StudentDocuments::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Students::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Students {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Name,
    Email,
    Phone,
    StudentNo,
    AadharCard,
    AbcId,
    Dob,
    MotherName,
    FathersName,
    Category,
    Handicapped,
    Address,
    Remarks,
    PhotoId,
}

#[derive(Iden)]
enum StudentDocuments {
    Table,
    StudentId,
    VNodeId,
}
