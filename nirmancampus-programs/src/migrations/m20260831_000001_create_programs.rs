use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProgramMedia::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProgramMedia::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProgramMedia::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ProgramMedia::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ProgramMedia::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(ProgramMedia::Language).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_program_media_deleted_at")
                    .table(ProgramMedia::Table)
                    .col(ProgramMedia::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Programs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Programs::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Programs::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Programs::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Programs::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Programs::Name).text())
                    .col(ColumnDef::new(Programs::Code).text().unique_key())
                    .col(ColumnDef::new(Programs::Description).text())
                    .col(
                        ColumnDef::new(Programs::University)
                            .string_len(32)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Programs::ProgramType)
                            .string_len(32)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Programs::AdmissionSessions)
                            .string_len(32)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Programs::TermType)
                            .string_len(32)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Programs::Fee)
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
                    .name("idx_programs_deleted_at")
                    .table(Programs::Table)
                    .col(Programs::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProgramStructureUnits::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProgramStructureUnits::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProgramStructureUnits::CreatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(ProgramStructureUnits::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(ProgramStructureUnits::DeletedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(ProgramStructureUnits::ProgramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProgramStructureUnits::TermNumber)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProgramStructureUnits::OptionalCourseCount).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_program_structure_units_program_id")
                            .from(ProgramStructureUnits::Table, ProgramStructureUnits::ProgramId)
                            .to(Programs::Table, Programs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_program_structure_units_deleted_at")
                    .table(ProgramStructureUnits::Table)
                    .col(ProgramStructureUnits::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .unique()
                    .name("idx_psu_program_term")
                    .table(ProgramStructureUnits::Table)
                    .col(ProgramStructureUnits::ProgramId)
                    .col(ProgramStructureUnits::TermNumber)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProgramProgramMedia::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProgramProgramMedia::ProgramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProgramProgramMedia::ProgramMediaId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ProgramProgramMedia::ProgramId)
                            .col(ProgramProgramMedia::ProgramMediaId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_program_program_media_program_id")
                            .from(ProgramProgramMedia::Table, ProgramProgramMedia::ProgramId)
                            .to(Programs::Table, Programs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_program_program_media_media_id")
                            .from(
                                ProgramProgramMedia::Table,
                                ProgramProgramMedia::ProgramMediaId,
                            )
                            .to(ProgramMedia::Table, ProgramMedia::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProgramStructureUnitCompulsoryCourses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProgramStructureUnitCompulsoryCourses::ProgramStructureUnitId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProgramStructureUnitCompulsoryCourses::CourseId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ProgramStructureUnitCompulsoryCourses::ProgramStructureUnitId)
                            .col(ProgramStructureUnitCompulsoryCourses::CourseId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_psu_compulsory_unit_id")
                            .from(
                                ProgramStructureUnitCompulsoryCourses::Table,
                                ProgramStructureUnitCompulsoryCourses::ProgramStructureUnitId,
                            )
                            .to(ProgramStructureUnits::Table, ProgramStructureUnits::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_psu_compulsory_course_id")
                            .from(
                                ProgramStructureUnitCompulsoryCourses::Table,
                                ProgramStructureUnitCompulsoryCourses::CourseId,
                            )
                            .to(Courses::Table, Courses::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProgramStructureUnitOptionalCourses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProgramStructureUnitOptionalCourses::ProgramStructureUnitId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProgramStructureUnitOptionalCourses::CourseId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ProgramStructureUnitOptionalCourses::ProgramStructureUnitId)
                            .col(ProgramStructureUnitOptionalCourses::CourseId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_psu_optional_unit_id")
                            .from(
                                ProgramStructureUnitOptionalCourses::Table,
                                ProgramStructureUnitOptionalCourses::ProgramStructureUnitId,
                            )
                            .to(ProgramStructureUnits::Table, ProgramStructureUnits::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_psu_optional_course_id")
                            .from(
                                ProgramStructureUnitOptionalCourses::Table,
                                ProgramStructureUnitOptionalCourses::CourseId,
                            )
                            .to(Courses::Table, Courses::Id)
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
                    .table(ProgramStructureUnitOptionalCourses::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ProgramStructureUnitCompulsoryCourses::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ProgramProgramMedia::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProgramStructureUnits::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Programs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProgramMedia::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ProgramMedia {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Language,
}

#[derive(Iden)]
enum Programs {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Name,
    Code,
    Description,
    University,
    ProgramType,
    AdmissionSessions,
    TermType,
    Fee,
}

#[derive(Iden)]
enum ProgramStructureUnits {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    ProgramId,
    TermNumber,
    OptionalCourseCount,
}

#[derive(Iden)]
enum ProgramProgramMedia {
    Table,
    ProgramId,
    ProgramMediaId,
}

#[derive(Iden)]
enum ProgramStructureUnitCompulsoryCourses {
    Table,
    ProgramStructureUnitId,
    CourseId,
}

#[derive(Iden)]
enum ProgramStructureUnitOptionalCourses {
    Table,
    ProgramStructureUnitId,
    CourseId,
}

#[derive(Iden)]
enum Courses {
    Table,
    Id,
}
