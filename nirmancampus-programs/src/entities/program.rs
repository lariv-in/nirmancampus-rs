use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "programs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub university: String,
    pub program_type: String,
    pub admission_sessions: String,
    pub term_type: String,
    pub fee: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::program_structure_unit::Entity")]
    StructureUnits,
    #[sea_orm(has_many = "super::program_program_media::Entity")]
    MediaLinks,
}

impl Related<super::program_structure_unit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StructureUnits.def()
    }
}

impl Related<super::program_program_media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaLinks.def()
    }
}

impl Related<super::program_media::Entity> for Entity {
    fn to() -> RelationDef {
        super::program_program_media::Relation::ProgramMedia.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::program_program_media::Relation::Program.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub type Program = Model;

impl Model {
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("")
    }

    pub fn code(&self) -> &str {
        self.code.as_deref().unwrap_or("")
    }

    pub fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }
}
