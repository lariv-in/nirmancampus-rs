use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "student_zone_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub title: String,
    pub is_link: Option<bool>,
    pub link: Option<String>,
    pub file_id: Option<i64>,
    pub student_zone_section_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::student_zone_section::Entity",
        from = "Column::StudentZoneSectionId",
        to = "super::student_zone_section::Column::Id",
        on_delete = "Cascade"
    )]
    Section,
}

impl Related<super::student_zone_section::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Section.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub type StudentZoneItem = Model;

impl Model {
    pub fn is_link(&self) -> bool {
        self.is_link.unwrap_or(false)
    }

    pub fn link(&self) -> &str {
        self.link.as_deref().unwrap_or("")
    }
}
