use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "student_application_documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub student_application_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub v_node_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::student_application::Entity",
        from = "Column::StudentApplicationId",
        to = "super::student_application::Column::Id",
        on_delete = "Cascade"
    )]
    Application,
}

impl Related<super::student_application::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Application.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
