use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "assignment_submissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub assignment_title: String,
    pub max_marks: i64,
    pub submission_status: String,
    pub marks: i64,
    pub course_id: i64,
    pub academic_record_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::assignment_submission_asset::Entity")]
    Assets,
}

impl Related<super::assignment_submission_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub type AssignmentSubmission = Model;

impl Model {
    pub fn assignment_title(&self) -> &str {
        &self.assignment_title
    }

    pub fn submission_status(&self) -> &str {
        &self.submission_status
    }
}
