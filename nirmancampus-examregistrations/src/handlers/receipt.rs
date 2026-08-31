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
use nirmancampus_common::{format_inr, program_display};

use crate::{
    entities::exam_registration::{self, Entity as ExamEntity},
    handlers::exams::{forbid_if_no_access, load_course, status_label},
    state::ExamRegistrationsState,
};

fn md_cell(s: &str) -> String {
    let t = s.replace('|', "\\|").trim().to_string();
    if t.is_empty() {
        "—".into()
    } else {
        t
    }
}

#[derive(Debug, Default, FromQueryResult)]
struct StudentLite {
    name: Option<String>,
    student_no: String,
}

#[derive(Debug, Default, FromQueryResult)]
struct RecordExtras {
    program_name: Option<String>,
    university: Option<String>,
    session_name: Option<String>,
    term_number: Option<i64>,
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
        "db find student for exam receipt",
    )
}

async fn load_record_extras(
    db: &sea_orm::DatabaseConnection,
    academic_record_id: i64,
) -> Option<RecordExtras> {
    lariv_rs::web::opt_or_log(
        RecordExtras::find_by_statement(sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            match db.get_database_backend() {
                sea_orm::DatabaseBackend::Postgres => {
                    "SELECT p.name AS program_name, p.university AS university, s.name AS session_name, u.term_number AS term_number
                     FROM academic_records r
                     LEFT JOIN programs p ON p.id = r.program_id
                     LEFT JOIN admission_sessions s ON s.id = r.session_id
                     LEFT JOIN program_structure_units u ON u.id = r.program_structure_unit_id
                     WHERE r.id = $1"
                }
                _ => {
                    "SELECT p.name AS program_name, p.university AS university, s.name AS session_name, u.term_number AS term_number
                     FROM academic_records r
                     LEFT JOIN programs p ON p.id = r.program_id
                     LEFT JOIN admission_sessions s ON s.id = r.session_id
                     LEFT JOIN program_structure_units u ON u.id = r.program_structure_unit_id
                     WHERE r.id = ?"
                }
            },
            [academic_record_id.into()],
        ))
        .one(db)
        .await,
        "db find exam receipt extras",
    )
}

pub fn exam_receipt_markdown(
    issued: &str,
    student_name: &str,
    student_no: &str,
    academic_record_id: i64,
    program: &str,
    session: &str,
    term: &str,
    rows: &[(String, String, String, String, String, String)],
) -> String {
    let mut table =
        String::from("| Exam | Course | Status | Fee | Marks | Recorded |\n|---|---|---|---|---|---|\n");
    for (title, course, status, fee, marks, recorded) in rows {
        table.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            md_cell(title),
            md_cell(course),
            md_cell(status),
            md_cell(fee),
            md_cell(marks),
            md_cell(recorded),
        ));
    }
    if rows.is_empty() {
        table = "_No exam registration rows on file for this academic record._\n".into();
    }
    format!(
        r#"# Exam registration acknowledgement

**Issued:** {issued}

## Student and academic record

| Field | Value |
|---|---|
| Name | {name} |
| Enrolment no. | {no} |
| Academic record id | {id} |
| Program | {program} |
| Session | {session} |
| Term | {term} |

## Registrations on file

{table}

---

The institution acknowledges that the exam registration entries listed above are **on record** for this academic period. File attachments, when applicable, remain stored per institutional policy.
"#,
        issued = md_cell(issued),
        name = md_cell(student_name),
        no = md_cell(student_no),
        id = academic_record_id,
        program = md_cell(program),
        session = md_cell(session),
        term = md_cell(term),
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

async fn record_visible(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    auth: &lariv_rs::plugins::users::state::AuthContext,
) -> Option<nirmancampus_academicrecords::entities::academic_record::Model> {
    use nirmancampus_academicrecords::entities::academic_record::{
        self, Entity as AcademicRecordEntity,
    };
    use nirmancampus_common::{is_admin, is_student};
    use sea_orm::sea_query::Expr;

    let mut query = AcademicRecordEntity::find_by_id(id)
        .filter(academic_record::Column::DeletedAt.is_null());
    if is_admin(auth) {
        return lariv_rs::web::opt_or_log(query.one(db).await, "db find academic record");
    }
    if is_student(auth) {
        let email = auth.user.email.trim().to_string();
        if email.is_empty() {
            return None;
        }
        query = query.filter(Expr::cust_with_values(
            "student_id IN (SELECT id FROM students WHERE email = ? AND deleted_at IS NULL)",
            [email],
        ));
        return lariv_rs::web::opt_or_log(query.one(db).await, "db find academic record");
    }
    None
}

pub async fn download(
    Cap(state): Cap<ExamRegistrationsState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(resp) = forbid_if_no_access(&ctx) {
        return resp;
    }
    let Some(rec) = record_visible(&state.db, id, &ctx).await else {
        return Redirect::to("/exam-registrations/").into_response();
    };
    let student = load_student_lite(&state.db, rec.student_id).await;
    let (name, no) = match &student {
        Some(s) => (s.name.clone().unwrap_or_default(), s.student_no.clone()),
        None => (String::new(), String::new()),
    };
    let extras = load_record_extras(&state.db, rec.id).await.unwrap_or_default();
    let program = program_display(
        extras.program_name.as_deref().unwrap_or(""),
        extras.university.as_deref().unwrap_or(""),
    );
    let session = extras.session_name.unwrap_or_default();
    let term = extras
        .term_number
        .map(|n| n.to_string())
        .unwrap_or_default();
    let rows_src = ExamEntity::find()
        .filter(exam_registration::Column::DeletedAt.is_null())
        .filter(exam_registration::Column::AcademicRecordId.eq(rec.id))
        .order_by_desc(exam_registration::Column::CreatedAt)
        .order_by_desc(exam_registration::Column::Id)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(rows_src.len());
    for e in &rows_src {
        let course_name = load_course(&state.db, e.course_id)
            .await
            .map(|c| c.name().to_string())
            .unwrap_or_default();
        let recorded = e
            .created_at
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        rows.push((
            e.exam_title().to_string(),
            course_name,
            status_label(&e.registration_status),
            format_inr(e.fee),
            format!("{} / {}", e.marks, e.max_marks),
            recorded,
        ));
    }
    let issued = ctx.format_datetime(Utc::now()).into_string();
    let md = exam_receipt_markdown(
        &issued, &name, &no, rec.id, &program, &session, &term, &rows,
    );
    let base = if no.trim().is_empty() {
        format!("exam-receipt-record-{}", rec.id)
    } else {
        format!("{no}-exams-{}", rec.id)
    };
    match export_pdf(&md).await {
        Ok(bytes) => file_response("application/pdf", &attachment_filename(&base, "pdf"), bytes),
        Err(e) => {
            tracing::warn!(error = %e, id = rec.id, "PDF export failed; falling back to markdown");
            file_response(
                "text/markdown; charset=utf-8",
                &attachment_filename(&base, "md"),
                md.into_bytes(),
            )
        }
    }
}
