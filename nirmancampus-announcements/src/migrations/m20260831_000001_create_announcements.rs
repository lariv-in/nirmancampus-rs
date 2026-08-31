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
                    .table(Announcements::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Announcements::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Announcements::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Announcements::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Announcements::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Announcements::Title).text().not_null())
                    .col(ColumnDef::new(Announcements::Description).text().not_null())
                    .col(ColumnDef::new(Announcements::Url).text())
                    .col(ColumnDef::new(Announcements::CreatedById).big_integer())
                    .col(ColumnDef::new(Announcements::ReleaseAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Announcements::ExpiryAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_announcements_created_by")
                            .from(Announcements::Table, Announcements::CreatedById)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_announcements_deleted_at")
                    .table(Announcements::Table)
                    .col(Announcements::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AnnouncementAssets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AnnouncementAssets::AnnouncementId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AnnouncementAssets::VNodeId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(AnnouncementAssets::AnnouncementId)
                            .col(AnnouncementAssets::VNodeId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_announcement_assets_announcement")
                            .from(AnnouncementAssets::Table, AnnouncementAssets::AnnouncementId)
                            .to(Announcements::Table, Announcements::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_announcement_assets_vnode")
                            .from(AnnouncementAssets::Table, AnnouncementAssets::VNodeId)
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
            .drop_table(Table::drop().table(AnnouncementAssets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Announcements::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Announcements {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Title,
    Description,
    Url,
    CreatedById,
    ReleaseAt,
    ExpiryAt,
}

#[derive(Iden)]
enum AnnouncementAssets {
    Table,
    AnnouncementId,
    VNodeId,
}
