use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tblfee")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub adm_session: String,
    pub adm_year: String,
    pub dod: Option<NaiveDate>,
    pub submit: String,
    pub prog: String,
    pub enroll: String,
    pub student: String,
    pub year_sem: String,
    pub category: String,
    pub dob: String,
    pub contact: String,
    pub deposit: String,
    pub nsd: String,
    pub fee: String,
    pub courses: String,
    pub remarks: String,
    pub deposit_by: String,
    pub ts: String,
    pub medium: String,
    pub mother_name: String,
    pub father_name: String,
    pub username: String,
    pub control_id: String,
    pub descrepency: String,
    pub university: String,
    pub payment_mode: String,
    pub trans_id: String,
    pub bank: String,
    pub rm: String,
    pub is_reconciled: String,
    pub online_exported: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Tblfee = Model;

impl Model {
    pub fn session_with_year(&self) -> String {
        format!("{} {}", self.adm_session.trim(), self.adm_year.trim())
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn dod_display(&self) -> String {
        self.dod
            .map(|d| d.format("%d-%m-%Y").to_string())
            .unwrap_or_default()
    }
}
