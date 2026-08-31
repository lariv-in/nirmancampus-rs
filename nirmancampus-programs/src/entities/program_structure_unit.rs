use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "program_structure_units")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub program_id: i64,
    pub term_number: i64,
    pub optional_course_count: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::program::Entity",
        from = "Column::ProgramId",
        to = "super::program::Column::Id",
        on_delete = "Cascade"
    )]
    Program,
    #[sea_orm(has_many = "super::program_structure_unit_compulsory_course::Entity")]
    CompulsoryLinks,
    #[sea_orm(has_many = "super::program_structure_unit_optional_course::Entity")]
    OptionalLinks,
}

impl Related<super::program::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Program.def()
    }
}

impl Related<super::program_structure_unit_compulsory_course::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CompulsoryLinks.def()
    }
}

impl Related<super::program_structure_unit_optional_course::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OptionalLinks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub type ProgramStructureUnit = Model;

impl Model {
    pub fn optional_course_count(&self) -> i64 {
        self.optional_course_count.unwrap_or(0)
    }
}
