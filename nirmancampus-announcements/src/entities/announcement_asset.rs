use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "announcement_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub announcement_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub v_node_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::announcement::Entity",
        from = "Column::AnnouncementId",
        to = "super::announcement::Column::Id",
        on_delete = "Cascade"
    )]
    Announcement,
}

impl Related<super::announcement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Announcement.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
