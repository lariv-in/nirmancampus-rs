use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "exam_registrations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub exam_title: String,
    pub max_marks: i64,
    pub registration_status: String,
    pub marks: i64,
    pub fee: i64,
    pub course_id: i64,
    pub academic_record_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::exam_registration_asset::Entity")]
    Assets,
}

impl Related<super::exam_registration_asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub type ExamRegistration = Model;

impl Model {
    pub fn exam_title(&self) -> &str {
        &self.exam_title
    }

    pub fn registration_status(&self) -> &str {
        &self.registration_status
    }
}
