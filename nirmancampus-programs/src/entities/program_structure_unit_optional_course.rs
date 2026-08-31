use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "program_structure_unit_optional_courses")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub program_structure_unit_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub course_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::program_structure_unit::Entity",
        from = "Column::ProgramStructureUnitId",
        to = "super::program_structure_unit::Column::Id",
        on_delete = "Cascade"
    )]
    StructureUnit,
    #[sea_orm(
        belongs_to = "nirmancampus_courses::entities::course::Entity",
        from = "Column::CourseId",
        to = "nirmancampus_courses::entities::course::Column::Id",
        on_delete = "Cascade"
    )]
    Course,
}

impl Related<super::program_structure_unit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StructureUnit.def()
    }
}

impl Related<nirmancampus_courses::entities::course::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Course.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
