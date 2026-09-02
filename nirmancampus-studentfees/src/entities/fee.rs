use chrono::{NaiveDateTime, NaiveDate};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::parse::{format_dod, opt_str};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tblfee")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_name = "ID")]
    pub id: i32,
    #[sea_orm(column_name = "AdmSession")]
    pub adm_session: Option<String>,
    #[sea_orm(column_name = "AdmYear")]
    pub adm_year: Option<String>,
    #[sea_orm(column_name = "DOD")]
    pub dod: Option<NaiveDateTime>,
    #[sea_orm(column_name = "Submit")]
    pub submit: Option<String>,
    #[sea_orm(column_name = "Prog")]
    pub prog: Option<String>,
    #[sea_orm(column_name = "Enroll")]
    pub enroll: Option<String>,
    #[sea_orm(column_name = "Student")]
    pub student: Option<String>,
    #[sea_orm(column_name = "YearSem")]
    pub year_sem: Option<String>,
    #[sea_orm(column_name = "Category")]
    pub category: Option<String>,
    #[sea_orm(column_name = "DOB")]
    pub dob: Option<String>,
    #[sea_orm(column_name = "Contact")]
    pub contact: Option<String>,
    #[sea_orm(column_name = "Deposit")]
    pub deposit: Option<String>,
    #[sea_orm(column_name = "NSD")]
    pub nsd: Option<String>,
    #[sea_orm(column_name = "Fee")]
    pub fee: Option<String>,
    #[sea_orm(column_name = "Courses")]
    pub courses: Option<String>,
    #[sea_orm(column_name = "Remarks")]
    pub remarks: Option<String>,
    #[sea_orm(column_name = "DepositBy")]
    pub deposit_by: Option<String>,
    #[sea_orm(column_name = "TS")]
    pub ts: Option<String>,
    #[sea_orm(column_name = "medium")]
    pub medium: Option<String>,
    #[sea_orm(column_name = "mother")]
    pub mother_name: Option<String>,
    #[sea_orm(column_name = "father")]
    pub father_name: Option<String>,
    #[sea_orm(column_name = "username")]
    pub username: Option<String>,
    #[sea_orm(column_name = "controlid")]
    pub control_id: Option<String>,
    #[sea_orm(column_name = "descrepency")]
    pub descrepency: Option<String>,
    #[sea_orm(column_name = "University")]
    pub university: Option<String>,
    #[sea_orm(column_name = "PaymentMode")]
    pub payment_mode: Option<String>,
    #[sea_orm(column_name = "TransID")]
    pub trans_id: Option<String>,
    #[sea_orm(column_name = "Bank")]
    pub bank: Option<String>,
    #[sea_orm(column_name = "RM")]
    pub rm: Option<String>,
    #[sea_orm(column_name = "IsReconciled")]
    pub is_reconciled: i8,
    #[sea_orm(column_name = "OnlineExported")]
    pub online_exported: i8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Fee = Model;

impl Model {
    pub fn text(value: &Option<String>) -> &str {
        opt_str(value)
    }

    pub fn session_with_year(&self) -> String {
        format!(
            "{} {}",
            opt_str(&self.adm_session).trim(),
            opt_str(&self.adm_year).trim()
        )
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    }

    pub fn dod_display(&self) -> String {
        format_dod(self.dod)
    }

    pub fn dod_date(&self) -> Option<NaiveDate> {
        self.dod.map(|d| d.date())
    }
}
