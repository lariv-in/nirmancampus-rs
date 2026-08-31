use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "assignment_submission_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub assignment_submission_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub v_node_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assignment_submission::Entity",
        from = "Column::AssignmentSubmissionId",
        to = "super::assignment_submission::Column::Id",
        on_delete = "Cascade"
    )]
    AssignmentSubmission,
}

impl Related<super::assignment_submission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AssignmentSubmission.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
