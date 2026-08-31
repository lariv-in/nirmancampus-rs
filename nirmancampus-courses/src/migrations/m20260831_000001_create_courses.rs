use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Courses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Courses::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Courses::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Courses::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Courses::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Courses::Name).text())
                    .col(ColumnDef::new(Courses::IsActive).boolean())
                    .col(ColumnDef::new(Courses::Code).text().unique_key())
                    .col(
                        ColumnDef::new(Courses::CourseType)
                            .string_len(64)
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(Courses::Description).text())
                    .col(
                        ColumnDef::new(Courses::Fee)
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
                    .name("idx_courses_deleted_at")
                    .table(Courses::Table)
                    .col(Courses::DeletedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Courses::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Courses {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Name,
    IsActive,
    Code,
    CourseType,
    Description,
    Fee,
}
