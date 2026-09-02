//! Public student-zone lookup helpers shared with the website plugin.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::{
    entities::fee::{self, Entity as FeeEntity, Model as FeeModel},
    parse::opt_str,
    state::StudentFeesState,
};

#[derive(Clone, Debug)]
pub struct StudentFeeView {
    pub session: String,
    pub receipt_id: String,
    pub name: String,
    pub dob: String,
    pub category: String,
    pub father_name: String,
    pub mobile: String,
    pub enrollment: String,
    pub program_code: String,
    pub courses: String,
    pub date_of_deposit: String,
    pub submit_type: String,
}

impl StudentFeeView {
    pub fn from_model(row: &FeeModel) -> Self {
        Self {
            session: opt_str(&row.adm_session).trim().to_string(),
            receipt_id: row.id.to_string(),
            name: opt_str(&row.student).to_string(),
            dob: mask_dob_year(opt_str(&row.dob)),
            category: opt_str(&row.category).to_string(),
            father_name: opt_str(&row.father_name).to_string(),
            mobile: mask_mobile(opt_str(&row.contact)),
            enrollment: opt_str(&row.enroll).to_string(),
            program_code: opt_str(&row.prog).to_string(),
            courses: opt_str(&row.courses).to_string(),
            date_of_deposit: row.dod_display(),
            submit_type: opt_str(&row.submit).to_string(),
        }
    }
}

pub fn digits_only(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub fn contact_matches(stored: &str, userid: &str) -> bool {
    let user = userid.trim();
    if user.is_empty() {
        return false;
    }
    if stored.trim() == user {
        return true;
    }
    let a = digits_only(stored);
    let b = digits_only(user);
    if a.is_empty() || b.is_empty() {
        return false;
    }
    if a == b {
        return true;
    }
    let a10 = if a.len() >= 10 {
        &a[a.len() - 10..]
    } else {
        a.as_str()
    };
    let b10 = if b.len() >= 10 {
        &b[b.len() - 10..]
    } else {
        b.as_str()
    };
    a10 == b10 && a10.len() >= 10
}

pub fn mask_dob_year(dob: &str) -> String {
    let s = dob.trim();
    if s.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = if s.contains('-') {
        s.split('-').collect()
    } else if s.contains('/') {
        s.split('/').collect()
    } else {
        return s.to_string();
    };
    if parts.len() != 3 {
        return s.to_string();
    }
    if parts[0].len() == 4 && parts[0].chars().all(|c| c.is_ascii_digit()) {
        return format!("****-{}-{}", parts[1], parts[2]);
    }
    if parts[2].len() == 4 && parts[2].chars().all(|c| c.is_ascii_digit()) {
        let sep = if s.contains('-') { "-" } else { "/" };
        return format!("{}{sep}{}{sep}****", parts[0], parts[1]);
    }
    s.to_string()
}

pub fn mask_mobile(contact: &str) -> String {
    let raw = contact.trim();
    if raw.is_empty() {
        return String::new();
    }
    let digits = digits_only(raw);
    if digits.len() < 4 {
        return "*".repeat(raw.chars().count().max(1));
    }
    let visible = &digits[digits.len() - 4..];
    format!("{}{visible}", "*".repeat(digits.len().saturating_sub(4)))
}

pub async fn find_by_id(state: &StudentFeesState, id: i64) -> Option<FeeModel> {
    let id = i32::try_from(id).ok().filter(|n| *n > 0)?;
    let db = state.mysql().await.ok()?;
    lariv_rs::web::opt_or_log(FeeEntity::find_by_id(id).one(&db).await, "db find tblfee")
}

pub async fn find_by_enroll(state: &StudentFeesState, enroll: &str) -> Vec<FeeModel> {
    let Ok(db) = state.mysql().await else {
        return Vec::new();
    };
    FeeEntity::find()
        .filter(fee::Column::Enroll.eq(enroll))
        .order_by_desc(fee::Column::Id)
        .all(&db)
        .await
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_dob_year_dmy() {
        assert_eq!(mask_dob_year("07-02-1984"), "07-02-****");
        assert_eq!(mask_dob_year("07/02/1984"), "07/02/****");
        assert_eq!(mask_dob_year("1984-02-07"), "****-02-07");
    }

    #[test]
    fn masks_mobile_last_four() {
        assert_eq!(mask_mobile("9915636130"), "******6130");
        assert_eq!(mask_mobile("91 9915636130"), "********6130");
    }

    #[test]
    fn contact_matches_last_ten_digits() {
        assert!(contact_matches("9915636130", "9915636130"));
        assert!(contact_matches("919915636130", "9915636130"));
        assert!(!contact_matches("9915636130", "9915636131"));
        assert!(!contact_matches("9915636130", ""));
    }
}
