use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "program_program_media")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub program_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub program_media_id: i64,
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
    #[sea_orm(
        belongs_to = "super::program_media::Entity",
        from = "Column::ProgramMediaId",
        to = "super::program_media::Column::Id",
        on_delete = "Cascade"
    )]
    ProgramMedia,
}

impl Related<super::program::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Program.def()
    }
}

impl Related<super::program_media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProgramMedia.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
