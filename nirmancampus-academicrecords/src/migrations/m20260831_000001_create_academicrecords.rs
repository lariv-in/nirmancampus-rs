use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AcademicRecords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AcademicRecords::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AcademicRecords::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(AcademicRecords::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(AcademicRecords::DeletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(AcademicRecords::StudentId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AcademicRecords::ProgramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AcademicRecords::SessionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AcademicRecords::ProgramStructureUnitId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AcademicRecords::Date).date())
                    .col(
                        ColumnDef::new(AcademicRecords::Status)
                            .string_len(50)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_academic_records_student")
                            .from(AcademicRecords::Table, AcademicRecords::StudentId)
                            .to(Students::Table, Students::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_academic_records_program")
                            .from(AcademicRecords::Table, AcademicRecords::ProgramId)
                            .to(Programs::Table, Programs::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_academic_records_session")
                            .from(AcademicRecords::Table, AcademicRecords::SessionId)
                            .to(AdmissionSessions::Table, AdmissionSessions::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_academic_records_program_structure_unit")
                            .from(
                                AcademicRecords::Table,
                                AcademicRecords::ProgramStructureUnitId,
                            )
                            .to(ProgramStructureUnits::Table, ProgramStructureUnits::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_academic_records_deleted_at")
                    .table(AcademicRecords::Table)
                    .col(AcademicRecords::DeletedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_academic_records_student_id")
                    .table(AcademicRecords::Table)
                    .col(AcademicRecords::StudentId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_academic_records_program_id")
                    .table(AcademicRecords::Table)
                    .col(AcademicRecords::ProgramId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_academic_records_session_id")
                    .table(AcademicRecords::Table)
                    .col(AcademicRecords::SessionId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_academic_records_program_structure_unit_id")
                    .table(AcademicRecords::Table)
                    .col(AcademicRecords::ProgramStructureUnitId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AcademicRecordCompulsoryCourses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AcademicRecordCompulsoryCourses::AcademicRecordId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AcademicRecordCompulsoryCourses::CourseId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(AcademicRecordCompulsoryCourses::AcademicRecordId)
                            .col(AcademicRecordCompulsoryCourses::CourseId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_academic_record_compulsory_courses_record")
                            .from(
                                AcademicRecordCompulsoryCourses::Table,
                                AcademicRecordCompulsoryCourses::AcademicRecordId,
                            )
                            .to(AcademicRecords::Table, AcademicRecords::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_academic_record_compulsory_courses_course")
                            .from(
                                AcademicRecordCompulsoryCourses::Table,
                                AcademicRecordCompulsoryCourses::CourseId,
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
                    .table(AcademicRecordOptionalCourses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AcademicRecordOptionalCourses::AcademicRecordId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AcademicRecordOptionalCourses::CourseId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(AcademicRecordOptionalCourses::AcademicRecordId)
                            .col(AcademicRecordOptionalCourses::CourseId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_academic_record_optional_courses_record")
                            .from(
                                AcademicRecordOptionalCourses::Table,
                                AcademicRecordOptionalCourses::AcademicRecordId,
                            )
                            .to(AcademicRecords::Table, AcademicRecords::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_academic_record_optional_courses_course")
                            .from(
                                AcademicRecordOptionalCourses::Table,
                                AcademicRecordOptionalCourses::CourseId,
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
                    .table(AcademicRecordOptionalCourses::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AcademicRecordCompulsoryCourses::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(AcademicRecords::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AcademicRecords {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    StudentId,
    ProgramId,
    SessionId,
    ProgramStructureUnitId,
    Date,
    Status,
}

#[derive(Iden)]
enum AcademicRecordCompulsoryCourses {
    Table,
    AcademicRecordId,
    CourseId,
}

#[derive(Iden)]
enum AcademicRecordOptionalCourses {
    Table,
    AcademicRecordId,
    CourseId,
}

#[derive(Iden)]
enum Students {
    Table,
    Id,
}

#[derive(Iden)]
enum Programs {
    Table,
    Id,
}

#[derive(Iden)]
enum AdmissionSessions {
    Table,
    Id,
}

#[derive(Iden)]
enum ProgramStructureUnits {
    Table,
    Id,
}

#[derive(Iden)]
enum Courses {
    Table,
    Id,
}
