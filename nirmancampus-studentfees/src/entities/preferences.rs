use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "student_fees_preferences")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type StudentFeesPreferences = Model;
