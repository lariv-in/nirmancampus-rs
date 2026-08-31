use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AdmissionSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AdmissionSessions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AdmissionSessions::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(AdmissionSessions::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(AdmissionSessions::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(AdmissionSessions::Name).text())
                    .col(ColumnDef::new(AdmissionSessions::Code).text().default(""))
                    .col(ColumnDef::new(AdmissionSessions::Start).timestamp_with_time_zone())
                    .col(ColumnDef::new(AdmissionSessions::End).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(AdmissionSessions::IsActive)
                            .boolean()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_admission_sessions_deleted_at")
                    .table(AdmissionSessions::Table)
                    .col(AdmissionSessions::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .unique()
                    .name("idx_admission_sessions_code")
                    .table(AdmissionSessions::Table)
                    .col(AdmissionSessions::Code)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AdmissionSessions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AdmissionSessions {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Name,
    Code,
    #[iden = "start"]
    Start,
    #[iden = "end"]
    End,
    IsActive,
}
