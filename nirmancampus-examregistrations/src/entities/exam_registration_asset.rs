use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "exam_registration_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub exam_registration_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub v_node_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::exam_registration::Entity",
        from = "Column::ExamRegistrationId",
        to = "super::exam_registration::Column::Id",
        on_delete = "Cascade"
    )]
    ExamRegistration,
}

impl Related<super::exam_registration::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExamRegistration.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
