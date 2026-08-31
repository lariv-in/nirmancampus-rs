use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "academic_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub student_id: i64,
    pub program_id: i64,
    pub session_id: i64,
    pub program_structure_unit_id: i64,
    pub date: Option<NaiveDate>,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::academic_record_compulsory_course::Entity")]
    CompulsoryLinks,
    #[sea_orm(has_many = "super::academic_record_optional_course::Entity")]
    OptionalLinks,
}

impl Related<super::academic_record_compulsory_course::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompulsoryLinks.def()
    }
}

impl Related<super::academic_record_optional_course::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OptionalLinks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub type AcademicRecord = Model;

impl Model {
    pub fn status(&self) -> &str {
        &self.status
    }
}
