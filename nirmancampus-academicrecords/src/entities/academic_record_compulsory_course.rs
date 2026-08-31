use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "academic_record_compulsory_courses")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub academic_record_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub course_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::academic_record::Entity",
        from = "Column::AcademicRecordId",
        to = "super::academic_record::Column::Id",
        on_delete = "Cascade"
    )]
    AcademicRecord,
}

impl Related<super::academic_record::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AcademicRecord.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
