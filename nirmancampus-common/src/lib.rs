//! Shared helpers for Nirmancampus plugins (roles, choices, scoping).

pub mod doc_export;
pub mod env;
pub mod schema;
pub mod ui;
pub mod user_names;
pub mod vnodes;

pub use user_names::{load_user_names, user_display};
pub use vnodes::{vnode_items, vnode_name, vnode_name_opt};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};

use lariv_rs::plugins::users::state::AuthContext;

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_STUDENT: &str = "student";
pub const ROLE_UNASSIGNED: &str = "unassigned";

/// Whether the user can see all records (admin or superuser).
pub fn is_admin(auth: &AuthContext) -> bool {
    auth.user.is_superuser || auth.role == ROLE_ADMIN
}

pub fn is_student(auth: &AuthContext) -> bool {
    auth.role == ROLE_STUDENT
}

pub fn is_unassigned(auth: &AuthContext) -> bool {
    auth.role == ROLE_UNASSIGNED
}

/// Admin/superuser or student — the campus record-viewing roles.
pub fn can_view_campus_records(auth: &AuthContext) -> bool {
    is_admin(auth) || is_student(auth)
}

pub fn optional_string(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

pub fn parse_optional_i64(s: &str) -> Option<i64> {
    let t = s.trim();
    if t.is_empty() || t == "0" {
        None
    } else {
        t.parse().ok()
    }
}

pub fn format_inr(amount: i64) -> String {
    format!("₹ {amount}")
}

pub fn format_inr_f64(amount: f64) -> String {
    format!("₹ {amount:.2}")
}

/// Go `ProgramDisplayLabel`: `"Name (University)"` when university is set.
pub fn program_display(name: &str, university: &str) -> String {
    if university.trim().is_empty() {
        return name.to_string();
    }
    let label = university_choice_pairs()
        .into_iter()
        .find(|(k, _)| k == university)
        .map(|(_, v)| v)
        .unwrap_or_else(|| university.to_string());
    format!("{name} ({label})")
}

pub fn category_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("GEN".into(), "General".into()),
        ("OBC".into(), "OBC".into()),
        ("SC".into(), "SC".into()),
        ("ST".into(), "ST".into()),
    ]
}

pub fn university_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("IGNOU".into(), "IGNOU".into()),
        ("MRSPTU".into(), "MRSPTU".into()),
    ]
}

pub fn program_type_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("bachelor".into(), "Bachelor".into()),
        ("certificate".into(), "Certificate".into()),
        ("diploma".into(), "Diploma".into()),
        ("masters".into(), "Masters".into()),
    ]
}

pub fn admission_session_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("both".into(), "January and July".into()),
        ("jan".into(), "January".into()),
        ("july".into(), "July".into()),
    ]
}

pub fn term_type_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("semester".into(), "Semester".into()),
        ("year".into(), "Year".into()),
    ]
}

pub fn academic_record_status_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("Not Applied".into(), "Not Applied".into()),
        ("Applied".into(), "Applied".into()),
        ("Enrolled".into(), "Enrolled".into()),
        ("Rejected".into(), "Rejected".into()),
    ]
}

pub fn payment_method_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("cash".into(), "Cash".into()),
        ("card".into(), "Card".into()),
        ("upi".into(), "UPI".into()),
        ("bank_transfer".into(), "Bank transfer".into()),
        ("cheque".into(), "Cheque".into()),
        ("other".into(), "Other".into()),
    ]
}

pub fn exam_status_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("not_registered".into(), "Not Registered".into()),
        ("registered".into(), "Registered".into()),
    ]
}

pub fn assignment_status_choice_pairs() -> Vec<(String, String)> {
    vec![
        ("not_marked".into(), "Not Marked".into()),
        ("marked".into(), "Marked".into()),
        ("uploaded".into(), "Uploaded".into()),
    ]
}

pub fn path_and_query(uri: &axum::http::Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

/// Apply created-by scoping for non-admin users on a SeaORM select.
pub fn scope_created_by<E>(query: Select<E>, auth: &AuthContext, column: E::Column) -> Select<E>
where
    E: EntityTrait,
    E::Column: ColumnTrait,
{
    if is_admin(auth) {
        query
    } else {
        query.filter(column.eq(auth.user.id))
    }
}
