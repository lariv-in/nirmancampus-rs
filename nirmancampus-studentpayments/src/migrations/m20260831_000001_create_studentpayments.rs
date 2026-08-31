use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Payments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Payments::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Payments::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Payments::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Payments::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Payments::StudentId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Payments::Amount)
                            .decimal_len(12, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Payments::PaymentMethod)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Payments::Remarks).text())
                    .col(
                        ColumnDef::new(Payments::TransactionId)
                            .string_len(255)
                            .default(""),
                    )
                    .col(ColumnDef::new(Payments::PaidAt).date())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_payments_student")
                            .from(Payments::Table, Payments::StudentId)
                            .to(Students::Table, Students::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_payments_deleted_at")
                    .table(Payments::Table)
                    .col(Payments::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_payments_student_id")
                    .table(Payments::Table)
                    .col(Payments::StudentId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Payments::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Payments {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    StudentId,
    Amount,
    PaymentMethod,
    Remarks,
    TransactionId,
    PaidAt,
}

#[derive(Iden)]
enum Students {
    Table,
    Id,
}
