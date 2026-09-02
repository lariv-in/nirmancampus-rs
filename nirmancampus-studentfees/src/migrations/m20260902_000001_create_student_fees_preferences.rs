use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StudentFeesPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StudentFeesPreferences::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StudentFeesPreferences::Host)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(StudentFeesPreferences::Port)
                            .integer()
                            .not_null()
                            .default(3306),
                    )
                    .col(
                        ColumnDef::new(StudentFeesPreferences::Username)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(StudentFeesPreferences::Password)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(StudentFeesPreferences::Database)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(StudentFeesPreferences::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum StudentFeesPreferences {
    Table,
    Id,
    Host,
    Port,
    Username,
    Password,
    Database,
}
