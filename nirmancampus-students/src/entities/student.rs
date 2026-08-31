use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "students")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[sea_orm(unique)]
    pub student_no: String,
    pub aadhar_card: Option<String>,
    pub abc_id: Option<String>,
    pub dob: Option<NaiveDate>,
    pub mother_name: Option<String>,
    pub fathers_name: Option<String>,
    pub category: Option<String>,
    pub handicapped: Option<bool>,
    #[sea_orm(column_type = "Text", nullable)]
    pub address: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub remarks: Option<String>,
    pub photo_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Student = Model;

impl Model {
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or("")
    }

    pub fn email(&self) -> &str {
        self.email.as_deref().unwrap_or("")
    }

    pub fn phone(&self) -> &str {
        self.phone.as_deref().unwrap_or("")
    }

    pub fn aadhar_card(&self) -> &str {
        self.aadhar_card.as_deref().unwrap_or("")
    }

    pub fn abc_id(&self) -> &str {
        self.abc_id.as_deref().unwrap_or("")
    }

    pub fn mother_name(&self) -> &str {
        self.mother_name.as_deref().unwrap_or("")
    }

    pub fn fathers_name(&self) -> &str {
        self.fathers_name.as_deref().unwrap_or("")
    }

    pub fn category(&self) -> &str {
        self.category.as_deref().unwrap_or("")
    }

    pub fn handicapped(&self) -> bool {
        self.handicapped.unwrap_or(false)
    }

    pub fn address(&self) -> &str {
        self.address.as_deref().unwrap_or("")
    }

    pub fn remarks(&self) -> &str {
        self.remarks.as_deref().unwrap_or("")
    }
}
