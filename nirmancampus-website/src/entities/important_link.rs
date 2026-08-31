use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "important_links")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub title: String,
    #[sea_orm(column_name = "order")]
    pub order: i64,
    pub is_link: Option<bool>,
    pub link: Option<String>,
    pub file_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type ImportantLink = Model;

impl Model {
    pub fn is_link(&self) -> bool {
        self.is_link.unwrap_or(false)
    }

    pub fn link(&self) -> &str {
        self.link.as_deref().unwrap_or("")
    }
}
