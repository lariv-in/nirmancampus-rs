use sea_orm_migration::prelude::*;

use nirmancampus_common::schema::filesystem_nodes_fk;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ImportantLinks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ImportantLinks::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ImportantLinks::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ImportantLinks::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ImportantLinks::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ImportantLinks::Title).text().not_null())
                    .col(
                        ColumnDef::new(ImportantLinks::Order)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(ImportantLinks::IsLink).boolean())
                    .col(ColumnDef::new(ImportantLinks::Link).text())
                    .col(ColumnDef::new(ImportantLinks::FileId).big_integer())
                    .foreign_key(&mut filesystem_nodes_fk(
                        "fk_important_links_file_id",
                        ImportantLinks::Table,
                        ImportantLinks::FileId,
                    ))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_important_links_deleted_at")
                    .table(ImportantLinks::Table)
                    .col(ImportantLinks::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ContactPageSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContactPageSettings::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ContactPageSettings::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ContactPageSettings::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ContactPageSettings::DeletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(ContactPageSettings::EssentialCommitteesListFileId)
                            .big_integer(),
                    )
                    .foreign_key(&mut filesystem_nodes_fk(
                        "fk_contact_page_settings_committees_file",
                        ContactPageSettings::Table,
                        ContactPageSettings::EssentialCommitteesListFileId,
                    ))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_contact_page_settings_deleted_at")
                    .table(ContactPageSettings::Table)
                    .col(ContactPageSettings::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(StudentZoneSections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentZoneSections::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(StudentZoneSections::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(StudentZoneSections::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(StudentZoneSections::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(StudentZoneSections::Title).text().not_null())
                    .col(
                        ColumnDef::new(StudentZoneSections::Order)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_student_zone_sections_deleted_at")
                    .table(StudentZoneSections::Table)
                    .col(StudentZoneSections::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(StudentZoneItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentZoneItems::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(StudentZoneItems::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(StudentZoneItems::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(StudentZoneItems::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(StudentZoneItems::Title).text().not_null())
                    .col(ColumnDef::new(StudentZoneItems::IsLink).boolean())
                    .col(ColumnDef::new(StudentZoneItems::Link).text())
                    .col(ColumnDef::new(StudentZoneItems::FileId).big_integer())
                    .col(ColumnDef::new(StudentZoneItems::StudentZoneSectionId).big_integer())
                    .foreign_key(&mut filesystem_nodes_fk(
                        "fk_student_zone_items_file_id",
                        StudentZoneItems::Table,
                        StudentZoneItems::FileId,
                    ))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_student_zone_items_section")
                            .from(
                                StudentZoneItems::Table,
                                StudentZoneItems::StudentZoneSectionId,
                            )
                            .to(StudentZoneSections::Table, StudentZoneSections::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_student_zone_items_deleted_at")
                    .table(StudentZoneItems::Table)
                    .col(StudentZoneItems::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_student_zone_items_student_zone_section_id")
                    .table(StudentZoneItems::Table)
                    .col(StudentZoneItems::StudentZoneSectionId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StudentZoneItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(StudentZoneSections::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ContactPageSettings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ImportantLinks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ImportantLinks {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Title,
    #[iden = "order"]
    Order,
    IsLink,
    Link,
    FileId,
}

#[derive(Iden)]
enum ContactPageSettings {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    EssentialCommitteesListFileId,
}

#[derive(Iden)]
enum StudentZoneSections {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Title,
    #[iden = "order"]
    Order,
}

#[derive(Iden)]
enum StudentZoneItems {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Title,
    IsLink,
    Link,
    FileId,
    StudentZoneSectionId,
}
