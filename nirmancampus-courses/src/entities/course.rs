use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "courses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub code: Option<String>,
    pub course_type: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub fee: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Course = Model;

impl Model {
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("")
    }

    pub fn code(&self) -> &str {
        self.code.as_deref().unwrap_or("")
    }

    pub fn is_active(&self) -> bool {
        self.is_active.unwrap_or(false)
    }

    pub fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }
}
