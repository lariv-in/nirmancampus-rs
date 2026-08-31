use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "student_applications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub program_id: i64,
    pub created_by_id: Option<i64>,
    pub student_name: String,
    pub email: Option<String>,
    pub dob: Option<NaiveDate>,
    pub mother_name: Option<String>,
    pub father_name: Option<String>,
    pub category: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub address: Option<String>,
    pub mobile: Option<String>,
    pub photo_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::student_application_document::Entity")]
    Documents,
}

impl Related<super::student_application_document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Documents.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub type StudentApplication = Model;

impl Model {
    pub fn student_name(&self) -> &str {
        &self.student_name
    }

    pub fn email(&self) -> &str {
        self.email.as_deref().unwrap_or("")
    }

    pub fn mother_name(&self) -> &str {
        self.mother_name.as_deref().unwrap_or("")
    }

    pub fn father_name(&self) -> &str {
        self.father_name.as_deref().unwrap_or("")
    }

    pub fn category(&self) -> &str {
        self.category.as_deref().unwrap_or("")
    }

    pub fn address(&self) -> &str {
        self.address.as_deref().unwrap_or("")
    }

    pub fn mobile(&self) -> &str {
        self.mobile.as_deref().unwrap_or("")
    }
}
