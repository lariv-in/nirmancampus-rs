use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "program_media")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub language: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::program_program_media::Entity")]
    ProgramLinks,
}

impl Related<super::program_program_media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProgramLinks.def()
    }
}

impl Related<super::program::Entity> for Entity {
    fn to() -> RelationDef {
        super::program_program_media::Relation::Program.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::program_program_media::Relation::ProgramMedia.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub type ProgramMedia = Model;
