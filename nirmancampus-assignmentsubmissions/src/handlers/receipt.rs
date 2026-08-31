use axum::{
    body::Body,
    extract::Path,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
};

use lariv_rs::{http::Cap, plugins::users::middleware::RequireAuth};
use nirmancampus_common::doc_export::{attachment_filename, export_pdf};

use crate::{
    entities::assignment_submission::{self, Entity as AssignmentEntity},
    handlers::assignments::{
        forbid_if_no_access, load_academic_record_visible, load_course, status_label,
    },
    state::AssignmentSubmissionsState,
};

fn md_cell(s: &str) -> String {
    let t = s.replace('|', "\\|").trim().to_string();
    if t.is_empty() { "—".into() } else { t }
}

#[derive(Debug, Default, FromQueryResult)]
struct StudentLite {
    name: Option<String>,
    student_no: String,
}

async fn load_student_lite(
    db: &sea_orm::DatabaseConnection,
    student_id: i64,
) -> Option<StudentLite> {
    if student_id <= 0 {
        return None;
    }
    lariv_rs::web::opt_or_log(
        StudentLite::find_by_statement(sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            match db.get_database_backend() {
                sea_orm::DatabaseBackend::Postgres => {
                    "SELECT name, student_no FROM students WHERE id = $1 AND deleted_at IS NULL"
                }
                _ => "SELECT name, student_no FROM students WHERE id = ? AND deleted_at IS NULL",
            },
            [student_id.into()],
        ))
        .one(db)
        .await,
        "db find student for assignment receipt",
    )
}

/// Build a markdown assignment-submission acknowledgement for PDF export.
pub fn assignment_receipt_markdown(
    issued: &str,
    student_name: &str,
    student_no: &str,
    academic_record_id: i64,
    rows: &[(String, String, String, i64, i64)],
) -> String {
    let mut table =
        String::from("| Assignment | Course | Marks | Max | Status |\n|---|---|---|---|---|\n");
    for (title, course, status, marks, max_marks) in rows {
        table.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            md_cell(title),
            md_cell(course),
            marks,
            max_marks,
            md_cell(status),
        ));
    }
    if rows.is_empty() {
        table = "_No assignment submission rows on file for this academic record._\n".into();
    }
    format!(
        r#"# Assignment submission acknowledgement

**Issued:** {issued}

## Student and academic record

| Field | Value |
|---|---|
| Name | {name} |
| Enrolment no. | {no} |
| Academic record id | {id} |

## Submissions on file

{table}

---

The institution acknowledges that the assignment submission entries listed above are **on record** for this academic period. File attachments, when applicable, remain stored per institutional policy.
"#,
        issued = md_cell(issued),
        name = md_cell(student_name),
        no = md_cell(student_no),
        id = academic_record_id,
        table = table,
    )
}

fn file_response(content_type: &str, filename: &str, bytes: Vec<u8>) -> Response {
    let mut resp = Response::new(Body::from(bytes));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type)
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        resp.headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    resp
}

pub async fn download(
    Cap(state): Cap<AssignmentSubmissionsState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let Some(rec) = load_academic_record_visible(&state.db, id, &ctx).await else {
        return Redirect::to("/assignment-submissions/").into_response();
    };
    let student = load_student_lite(&state.db, rec.student_id).await;
    let (name, no) = match &student {
        Some(s) => (s.name.clone().unwrap_or_default(), s.student_no.clone()),
        None => (String::new(), String::new()),
    };
    let rows_src = AssignmentEntity::find()
        .filter(assignment_submission::Column::DeletedAt.is_null())
        .filter(assignment_submission::Column::AcademicRecordId.eq(rec.id))
        .order_by_desc(assignment_submission::Column::CreatedAt)
        .order_by_desc(assignment_submission::Column::Id)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(rows_src.len());
    for s in &rows_src {
        let course_name = load_course(&state.db, s.course_id)
            .await
            .map(|c| c.name().to_string())
            .unwrap_or_default();
        rows.push((
            s.assignment_title().to_string(),
            course_name,
            status_label(&s.submission_status),
            s.marks,
            s.max_marks,
        ));
    }
    let issued = ctx.format_datetime(Utc::now()).into_string();
    let md = assignment_receipt_markdown(&issued, &name, &no, rec.id, &rows);
    let base = if no.trim().is_empty() {
        format!("assignment-receipt-record-{}", rec.id)
    } else {
        format!("{no}-assignments-{}", rec.id)
    };
    match export_pdf(&md).await {
        Ok(bytes) => file_response("application/pdf", &attachment_filename(&base, "pdf"), bytes),
        Err(e) => {
            tracing::warn!(error = %e, id, "PDF export failed; falling back to markdown");
            file_response(
                "text/markdown; charset=utf-8",
                &attachment_filename(&base, "md"),
                md.into_bytes(),
            )
        }
    }
}
