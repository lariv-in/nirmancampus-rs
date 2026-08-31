use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub student_id: i64,
    #[sea_orm(column_type = "Decimal(Some((12, 2)))")]
    pub amount: Decimal,
    pub payment_method: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub remarks: Option<String>,
    pub transaction_id: Option<String>,
    pub paid_at: Option<NaiveDate>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Payment = Model;

impl Model {
    pub fn payment_method(&self) -> &str {
        &self.payment_method
    }

    pub fn remarks(&self) -> &str {
        self.remarks.as_deref().unwrap_or("")
    }

    pub fn transaction_id(&self) -> &str {
        self.transaction_id.as_deref().unwrap_or("")
    }
}
