use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "admission_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub start: Option<DateTime<Utc>>,
    #[sea_orm(column_name = "end")]
    pub end: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type AdmissionSession = Model;

impl Model {
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("")
    }

    pub fn code(&self) -> &str {
        self.code.as_deref().unwrap_or("")
    }

    pub fn is_active(&self) -> bool {
        self.is_active.unwrap_or(true)
    }
}
